use std::collections::HashMap;
use std::fmt;

use anyhow::*;

use crate::{
    entities::{DocumentData, Id},
    schema::{DataDescription, Field},
};

use super::db::{ArhivTransaction, Queries};

pub type FieldValidationErrors = HashMap<String, Vec<Error>>;

#[derive(Debug)]
pub struct ValidationError {
    pub errors: FieldValidationErrors,
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn count_errors(&self) -> usize {
        self.errors
            .values()
            .fold(0, |total, field_errors| total + field_errors.len())
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Document validation error")?;

        for (field, errors) in &self.errors {
            writeln!(f, "  field '{}': {} errors", field, errors.len())?;
            for error in errors {
                writeln!(f, "                {}", error)?;
            }
        }

        writeln!(f, "{} errors in total", self.count_errors())?;

        Ok(())
    }
}

#[derive(Default)]
pub struct Validator {
    valid_refs: HashMap<Id, String>,
    errors: HashMap<String, Vec<Error>>,
}

impl Validator {
    fn validate_ref(
        &mut self,
        id: &Id,
        expected_document_type: Option<&str>,
        tx: &mut ArhivTransaction<'_>,
    ) -> Result<()> {
        match (self.valid_refs.get(id), expected_document_type) {
            (Some(_document_type), None) => {
                return Ok(());
            }
            (Some(document_type), Some(expected_document_type)) => {
                if document_type == expected_document_type {
                    return Ok(());
                }

                bail!(
                    "document '{}' expected to be '{}' but has type '{}'",
                    id,
                    expected_document_type,
                    document_type
                );
            }
            _ => {}
        };

        let document = if let Some(document) = tx.get_document(id)? {
            document
        } else {
            bail!("unknown document ref '{}'", id);
        };

        self.valid_refs
            .insert(document.id, document.document_type.clone());

        if let Some(document_type) = expected_document_type {
            ensure!(
                document.document_type == document_type,
                "document '{}' expected to be '{}' but has type '{}'",
                id,
                document_type,
                document.document_type,
            );
        }

        Ok(())
    }

    fn track_err<T>(&mut self, field: &Field, result: Result<T>) {
        if let Err(err) = result {
            self.errors
                .entry(field.name.to_string())
                .or_default()
                .push(err);
        }
    }

    pub fn validate(
        mut self,
        data: &DocumentData,
        prev_data: Option<&DocumentData>,
        data_description: &DataDescription,
        tx: &mut ArhivTransaction<'_>,
    ) -> std::result::Result<(), ValidationError> {
        for field in &data_description.fields {
            // first, validate field value
            let value = data.get(field.name);
            let prev_value = prev_data
                .as_ref()
                .and_then(|prev_data| prev_data.get(field.name));

            let validation_result = field.validate(value, prev_value);

            if validation_result.is_err() {
                self.track_err(field, validation_result);
                continue;
            }

            let refs = {
                if let Some(value) = value {
                    field.get_refs(value)
                } else {
                    continue;
                }
            };

            let expected_document_type = field.get_expected_ref_type();

            // then check field refs
            for id in refs {
                let validation_result = self.validate_ref(&id, expected_document_type, tx);

                self.track_err(field, validation_result);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError {
                errors: self.errors,
            })
        }
    }
}
