use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, bail, ensure, Result};

use crate::{
    baza2::Baza,
    entities::{BLOBId, Document, Id},
    schema::{DataSchema, Field},
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

// FIXME refactor this - remove unnecessary trait
pub trait ValidableDB {
    fn get_schema(&self) -> &DataSchema;

    fn get_document(&self, id: &Id) -> Result<Option<Document>>;

    fn blob_exists(&self, blob_id: &BLOBId) -> Result<bool>;
}

impl ValidableDB for &Baza {
    fn get_schema(&self) -> &DataSchema {
        (self as &Baza).get_schema()
    }

    fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let document = (self as &Baza)
            .get_document(id)
            .map(|head| head.get_single_document().clone()); // FIXME remove clone

        Ok(document)
    }

    fn blob_exists(&self, blob_id: &BLOBId) -> Result<bool> {
        (self as &Baza).blob_exists(blob_id)
    }
}

pub struct Validator<DB: ValidableDB> {
    db: DB,
    errors: FieldValidationErrors,
}

impl<DB: ValidableDB> Validator<DB> {
    pub fn new(db: DB) -> Self {
        Validator {
            db,
            errors: HashMap::new(),
        }
    }

    fn validate_ref(&self, id: &Id, expected_document_types: Option<&[&str]>) -> Result<()> {
        let document = if let Some(document) = self.db.get_document(id)? {
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
        let blob_exists = self.db.blob_exists(blob_id)?;

        ensure!(blob_exists, "BLOB {blob_id} doesn't exist");

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
        let schema = self.db.get_schema();

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
        prev_document: &Option<Document>,
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
    ) -> std::result::Result<(), ValidationError> {
        let prev_document = self.db.get_document(&document.id)?;

        self.validate_document_attributes(document, &prev_document)?;

        self.validate_fields_presence(document)?;

        let schema = self.db.get_schema().clone();
        for field in schema.iter_fields(&document.document_type)? {
            let value = document.data.get(field.name);

            // ensure readonly field didn't change
            if let Some(ref prev_document) = prev_document {
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

// FIXME fix tests
// #[cfg(test)]
// mod tests {
//     use serde_json::json;

//     use crate::{
//         schema::{DataDescription, DataSchema, Field, FieldType},
//         tests::new_document,
//     };

//     #[test]
//     fn test_validation_mandatory() {
//         let baza = Baza::new_with_schema(DataSchema::new(
//             "test",
//             vec![DataDescription {
//                 document_type: "test_type",
//                 title_format: "title",
//                 fields: vec![Field {
//                     name: "test",
//                     field_type: FieldType::String {},
//                     mandatory: true,
//                     readonly: false,
//                 }],
//             }],
//         ));

//         {
//             let mut tx = baza.get_tx().unwrap();
//             let mut document = new_document(json!({}));
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_err());
//         }

//         {
//             let mut tx = baza.get_tx().unwrap();
//             let mut document = new_document(json!({ "test": "test" }));
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_ok());
//         }
//     }

//     #[test]
//     fn test_validation_readonly() {
//         let baza = Baza::new_with_schema(DataSchema::new(
//             "test",
//             vec![DataDescription {
//                 document_type: "test_type",
//                 title_format: "title",
//                 fields: vec![Field {
//                     name: "test",
//                     field_type: FieldType::String {},
//                     mandatory: false,
//                     readonly: true,
//                 }],
//             }],
//         ));

//         {
//             let mut tx = baza.get_tx().unwrap();

//             let mut document = new_document(json!({ "test": "test" }));
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_ok());

//             document.data = json!({ "test": None::<String> }).try_into().unwrap();
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_err());
//         }

//         {
//             let mut tx = baza.get_tx().unwrap();

//             let mut document = new_document(json!({}));
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_ok());

//             document.data = json!({ "test": "test" }).try_into().unwrap();
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_err());

//             document.data = json!({ "test": None::<String> }).try_into().unwrap();
//             let result = tx.stage_document(&mut document, None);
//             assert!(result.is_ok());
//         }
//     }
// }
