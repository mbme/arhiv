use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, bail, ensure, Error, Result};

use crate::{
    arhiv::db::ArhivConnection,
    entities::{BLOBId, DocumentData, Id},
    schema::{DataDescription, Field},
};

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
    errors: HashMap<String, Vec<Error>>,
}

impl Validator {
    fn validate_ref(
        id: &Id,
        expected_document_type: Option<&str>,
        tx: &ArhivConnection,
    ) -> Result<()> {
        let document = if let Some(document) = tx.get_document(id)? {
            document
        } else {
            bail!("unknown document ref '{}'", id);
        };

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

    fn validate_blob_id(blob_id: &BLOBId, tx: &ArhivConnection) -> Result<()> {
        let is_known_blob_id = tx.is_known_blob_id(blob_id)?;

        if is_known_blob_id {
            return Ok(());
        }

        // should be known OR MUST exist
        let blob = tx.get_blob(blob_id);

        ensure!(blob.exists()?, "new blob {} doesn't exist", blob_id);

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
        tx: &ArhivConnection,
    ) -> std::result::Result<(), ValidationError> {
        for field in &data_description.fields {
            let value = data.get(field.name);

            // ensure readonly field didn't change
            if let Some(prev_data) = prev_data {
                let prev_value = prev_data.get(field.name);

                if field.readonly && value != prev_value {
                    self.track_err::<()>(
                        field,
                        Err(anyhow!(
                            "value of readonly field '{}' changed from '{:?}' to '{:?}'",
                            field.name,
                            prev_value,
                            value,
                        )),
                    );
                    continue;
                }
            }

            // validate field value
            let validation_result = field.validate(value);

            if validation_result.is_err() {
                self.track_err(field, validation_result);
                continue;
            }

            // check blob ids
            if let Some(value) = value {
                let blob_ids = field.extract_blob_ids(value);

                for blob_id in blob_ids {
                    let validation_result = Validator::validate_blob_id(&blob_id, tx);

                    self.track_err(field, validation_result);
                }
            }

            let refs = {
                if let Some(value) = value {
                    field.extract_refs(value)
                } else {
                    continue;
                }
            };

            let expected_document_type = field.get_expected_ref_type();

            // then check field refs
            for id in refs {
                let validation_result = Validator::validate_ref(&id, expected_document_type, tx);

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
