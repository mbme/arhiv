use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, bail, ensure, Result};

use crate::{
    db::BazaConnection,
    entities::{BLOBId, Document, Id},
    schema::Field,
};

pub type FieldValidationErrors = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub enum ValidationError {
    FieldError { errors: FieldValidationErrors },
    DocumentError { errors: Vec<String> },
}

impl ValidationError {
    fn throw_document_error(errors: Vec<String>) -> std::result::Result<(), Self> {
        Err(ValidationError::DocumentError { errors })
    }
}

impl From<anyhow::Error> for ValidationError {
    fn from(value: anyhow::Error) -> Self {
        ValidationError::DocumentError {
            errors: vec![value.to_string()],
        }
    }
}

impl std::error::Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Document validation error")?;

        match self {
            ValidationError::FieldError { errors } => {
                for (field, errors) in errors {
                    writeln!(f, "  field '{}': {} errors", field, errors.len())?;
                    for error in errors {
                        writeln!(f, "                {error}")?;
                    }
                }

                let total_errors_count = errors
                    .values()
                    .fold(0, |total, field_errors| total + field_errors.len());

                writeln!(f, "{total_errors_count} errors in total")?;
            }
            ValidationError::DocumentError { errors } => {
                for error in errors {
                    writeln!(f, "{error}")?;
                }
            }
        }

        Ok(())
    }
}

pub struct Validator<'c> {
    conn: &'c BazaConnection,
    errors: FieldValidationErrors,
}

impl<'c> Validator<'c> {
    pub fn new(conn: &'c BazaConnection) -> Self {
        Validator {
            conn,
            errors: HashMap::new(),
        }
    }

    fn validate_ref(&self, id: &Id, expected_document_types: Option<&[&str]>) -> Result<()> {
        let document = if let Some(document) = self.conn.get_document(id)? {
            document
        } else {
            bail!("unknown document ref '{}'", id);
        };

        if let Some(document_types) = expected_document_types {
            if !document_types.is_empty() {
                ensure!(
                    document_types.contains(&document.document_type.as_ref()),
                    "document '{}' expected to be '{}' but has type '{}'",
                    id,
                    document_types.join(", "),
                    document.document_type,
                );
            }
        }

        Ok(())
    }

    fn validate_blob_id(&self, blob_id: &BLOBId) -> Result<()> {
        let is_known_blob_id = self.conn.is_known_blob_id(blob_id)?;

        if is_known_blob_id {
            return Ok(());
        }

        // should be known OR MUST exist
        let blob = self.conn.get_blob(blob_id);

        ensure!(blob.exists()?, "new blob {} doesn't exist", blob_id);

        Ok(())
    }

    fn track_err<T>(&mut self, field: &Field, result: Result<T>) {
        if let Err(err) = result {
            self.errors
                .entry(field.name.to_string())
                .or_default()
                .push(err.to_string());
        }
    }

    /// ensure fields should be present in document
    fn validate_fields_presence(
        &self,
        document: &Document,
    ) -> std::result::Result<(), ValidationError> {
        let schema = self.conn.get_schema();

        let data_description = schema.get_data_description(&document.document_type)?;

        let errors = document
            .data
            .iter_fields()
            .filter_map(|(field_name, value)| {
                if value.is_null() {
                    return None;
                }

                if let Some(_field) = data_description.get_field(field_name) {
                    return None;
                }

                Some(format!(
                    "Document type '{}' doesn't expect field '{}'",
                    &document.document_type, field_name
                ))
            })
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::DocumentError { errors })
        }
    }

    fn validate_document_attributes(
        &self,
        document: &Document,
        prev_document: Option<&Document>,
    ) -> std::result::Result<(), ValidationError> {
        let mut document_errors = Vec::with_capacity(3);
        if document.is_erased() {
            document_errors.push("Erased documents can't be staged".to_string());
        }

        if let Some(prev_document) = prev_document {
            if document.document_type != prev_document.document_type {
                document_errors.push(format!(
                    "document type '{}' is different from the type '{}' of existing document",
                    document.document_type, prev_document.document_type
                ));
            }

            if document.updated_at != prev_document.updated_at {
                document_errors.push(format!(
                "document updated_at '{}' is different from the updated_at '{}' of existing document",
                document.updated_at,
                prev_document.updated_at
                ));
            }
        }

        if document_errors.is_empty() {
            Ok(())
        } else {
            ValidationError::throw_document_error(document_errors)
        }
    }

    pub fn validate_staged(
        mut self,
        document: &Document,
        prev_document: Option<&Document>,
    ) -> std::result::Result<(), ValidationError> {
        self.validate_document_attributes(document, prev_document)?;

        self.validate_fields_presence(document)?;

        for field in self
            .conn
            .get_schema()
            .iter_fields(&document.document_type)?
        {
            let value = document.data.get(field.name);

            // ensure readonly field didn't change
            if let Some(prev_document) = prev_document {
                let prev_value = prev_document.data.get(field.name);

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
                    let validation_result = self.validate_blob_id(&blob_id);

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

            let expected_document_types = field.get_expected_ref_types();

            // then check field refs
            for id in refs {
                let validation_result = self.validate_ref(&id, expected_document_types);

                self.track_err(field, validation_result);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::FieldError {
                errors: self.errors,
            })
        }
    }
}
