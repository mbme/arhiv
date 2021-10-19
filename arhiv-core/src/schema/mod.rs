use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;

use crate::entities::{Document, DocumentData, Refs, ATTACHMENT_TYPE, TOMBSTONE_TYPE};

pub use data_description::*;
pub use field::*;

mod data_description;
mod field;
mod search;

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    modules: Vec<DataDescription>,
    internal_document_types: Vec<&'static str>,
}

impl DataSchema {
    #[must_use]
    pub fn new() -> DataSchema {
        let modules = vec![
            // ----- INTERNAL
            DataDescription {
                document_type: TOMBSTONE_TYPE,
                collection_of: Collection::None,
                fields: vec![],
            },
            DataDescription {
                document_type: ATTACHMENT_TYPE,
                collection_of: Collection::None,
                fields: vec![
                    Field {
                        name: "filename",
                        field_type: FieldType::String {},
                        mandatory: true,
                    },
                    Field {
                        name: "sha256",
                        field_type: FieldType::ReadonlyString {},
                        mandatory: true,
                    },
                ],
            },
            // ----
        ];

        DataSchema {
            internal_document_types: modules.iter().map(|module| module.document_type).collect(),
            modules,
        }
    }

    pub fn with_modules(&mut self, modules: &mut Vec<DataDescription>) {
        self.modules.append(modules);
    }

    fn get_collection_ref_fields(&self, document_type: &str) -> HashSet<&str> {
        self.modules
            .iter()
            .filter_map(|module| match module.collection_of {
                Collection::Type {
                    document_type: child_type,
                    field,
                } if child_type == document_type => Some(field),
                _ => None,
            })
            .collect()
    }

    pub fn extract_refs(&self, document_type: &str, data: &DocumentData) -> Result<Refs> {
        let data_description = self.get_data_description(document_type)?;

        let collection_ref_fields = self.get_collection_ref_fields(document_type);

        let mut refs = Refs::new();

        for field in &data_description.fields {
            let value = if let Some(value) = data.get(field.name) {
                value
            } else {
                continue;
            };

            if collection_ref_fields.contains(&field.name) {
                refs.collections.extend(field.get_refs(value));
            } else {
                refs.documents.extend(field.get_refs(value));
            }
        }

        Ok(refs)
    }

    pub fn get_data_description(&self, document_type: impl AsRef<str>) -> Result<&DataDescription> {
        let document_type = document_type.as_ref();

        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or_else(|| anyhow!("Unknown document type: {}", document_type))
    }

    pub fn get_title<'doc>(&self, document: &'doc Document) -> Result<&'doc str> {
        let data_description = self.get_data_description(&document.document_type)?;

        let title_field = if let Some(title_field) = data_description.pick_title_field() {
            title_field
        } else {
            return Ok("Untitled");
        };

        document
            .data
            .get_str(title_field.name)
            .ok_or_else(|| anyhow!("title field {} is missing", title_field.name))
    }

    #[must_use]
    pub fn get_document_types(&self, skip_internal: bool) -> Vec<&'static str> {
        self.modules
            .iter()
            .filter(|module| {
                if skip_internal {
                    !self.is_internal_type(module.document_type)
                } else {
                    true
                }
            })
            .map(|module| module.document_type)
            .collect()
    }

    #[must_use]
    pub fn is_internal_type(&self, document_type: &str) -> bool {
        self.internal_document_types.contains(&document_type)
    }
}

impl Default for DataSchema {
    fn default() -> Self {
        Self::new()
    }
}
