use std::collections::HashMap;

use anyhow::*;

use rs_utils::log;

use crate::{
    entities::{Document, Id},
    schema::FieldType,
    Arhiv,
};

pub struct Validator<'a> {
    arhiv: &'a Arhiv,
    valid_ids: HashMap<Id, String>,
    errors: Vec<Error>,
}

impl<'a> Validator<'a> {
    pub fn new(arhiv: &'a Arhiv) -> Self {
        Validator {
            arhiv,
            valid_ids: HashMap::new(),
            errors: vec![],
        }
    }

    fn validate_ref(&mut self, id: &Id, expected_document_type: Option<&str>) -> Result<()> {
        match (self.valid_ids.get(id), expected_document_type) {
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

        let document = if let Some(document) = self.arhiv.get_document(id)? {
            document
        } else {
            bail!("unknown document '{}'", id);
        };

        self.valid_ids
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

    fn track_err<T>(&mut self, result: Result<T>) {
        if let Err(err) = result {
            self.errors.push(err);
        }
    }

    fn build_validation_result(self) -> Result<()> {
        if !self.errors.is_empty() {
            bail!(
                "invalid document: {} errors\n{}",
                self.errors.len(),
                self.errors
                    .into_iter()
                    .map(|err| err.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        Ok(())
    }

    pub fn validate(mut self, document: &Document, prev_document: &Option<Document>) -> Result<()> {
        let data_description = self
            .arhiv
            .schema
            .get_data_description(&document.document_type)?;

        for field in &data_description.fields {
            let value = document.data.get(field.name);
            let prev_value = prev_document
                .as_ref()
                .and_then(|prev_document| prev_document.data.get(field.name));

            let validation_result = field.validate(value, prev_value);

            if validation_result.is_err() {
                self.track_err(validation_result);
                continue;
            }

            let refs = {
                if let Some(value) = value {
                    field.get_refs(value)
                } else {
                    continue;
                }
            };

            // check field refs
            for id in refs {
                if id == document.id {
                    log::warn!("Document {} references itself", &document.id);
                    continue;
                }

                let validation_result = self
                    .validate_ref(&id, None)
                    .context("refs validation failed");

                self.track_err(validation_result);
            }
        }

        // check document types of refs
        for field in &data_description.fields {
            match field.field_type {
                FieldType::Ref(document_type) => {
                    let id: Id = if let Some(id) = document.data.get_str(field.name) {
                        id.into()
                    } else {
                        continue;
                    };

                    let validation_result = self
                        .validate_ref(&id, Some(document_type))
                        .context(anyhow!("field '{}' validation failed", field.name));

                    self.track_err(validation_result);
                }

                FieldType::RefList(document_type) => {
                    let value = if let Some(value) = document.data.get(field.name) {
                        value
                    } else {
                        continue;
                    };

                    let ids: Vec<Id> = serde_json::from_value(value.clone())?;

                    for id in ids {
                        let validation_result = self
                            .validate_ref(&id, Some(document_type))
                            .context(anyhow!("field '{}' validation failed", field.name));

                        self.track_err(validation_result);
                    }
                }

                _ => {}
            }
        }

        self.build_validation_result()
    }
}
