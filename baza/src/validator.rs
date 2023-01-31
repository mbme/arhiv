use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, bail, ensure, Result};

use crate::{
    db::BazaConnection,
    entities::{BLOBId, Document, DocumentData, DocumentType, Id},
    schema::Field,
};

pub type FieldValidationErrors = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub enum ValidationError {
    FieldError { errors: FieldValidationErrors },
    DocumentError { errors: Vec<String> },
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
                        writeln!(f, "                {}", error)?;
                    }
                }

                let total_errors_count = errors
                    .values()
                    .fold(0, |total, field_errors| total + field_errors.len());

                writeln!(f, "{} errors in total", total_errors_count)?;
            }
            ValidationError::DocumentError { errors } => {
                for error in errors {
                    writeln!(f, "{}", error)?;
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

    fn validate_ref(&self, id: &Id, expected_document_type: Option<&str>) -> Result<()> {
        let document = if let Some(document) = self.conn.get_document(id)? {
            document
        } else {
            bail!("unknown document ref '{}'", id);
        };

        if let Some(document_type) = expected_document_type {
            ensure!(
                document.document_type.document_type == document_type,
                "document '{}' expected to be '{}' but has type '{}'",
                id,
                document_type,
                document.document_type,
            );
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

    fn validate_document_type(
        &self,
        document_type: &DocumentType,
    ) -> std::result::Result<(), ValidationError> {
        let schema = self.conn.get_schema();

        let data_description = schema.get_data_description(document_type).map_err(|err| {
            ValidationError::DocumentError {
                errors: vec![err.to_string()],
            }
        })?;

        if !data_description.is_supported_subtype(&document_type.subtype) {
            return Err(ValidationError::DocumentError {
                errors: vec![format!(
                    "Document type '{}' doesn't have subtype '{}'",
                    document_type.document_type, document_type.subtype
                )],
            });
        }

        Ok(())
    }

    /// ensure fields should be present in document / subtype
    fn validate_fields_presence(
        &self,
        document: &Document,
    ) -> std::result::Result<(), ValidationError> {
        let schema = self.conn.get_schema();

        let data_description = schema
            .get_data_description(&document.document_type)
            .expect("document_type must be valid");

        let errors = document
            .data
            .iter_fields()
            .filter_map(|(field_name, value)| {
                if value.is_null() {
                    return None;
                }

                if data_description
                    .get_field(field_name)
                    .map_or(false, |field| {
                        field.for_subtype(&document.document_type.subtype)
                    })
                {
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

    pub fn validate(
        mut self,
        document: &Document,
        prev_data: Option<&DocumentData>,
    ) -> std::result::Result<(), ValidationError> {
        self.validate_document_type(&document.document_type)?;

        self.validate_fields_presence(document)?;

        for field in self
            .conn
            .get_schema()
            .iter_fields(&document.document_type)
            .expect("document_type must be valid")
        {
            let value = document.data.get(field.name);

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

            let expected_document_type = field.get_expected_ref_type();

            // then check field refs
            for id in refs {
                let validation_result = self.validate_ref(&id, expected_document_type);

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
