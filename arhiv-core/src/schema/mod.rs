use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::entities::{Document, DocumentData, Refs, ERASED_DOCUMENT_TYPE};

pub use data_description::*;
pub use field::*;

mod data_description;
mod field;
mod search;

const ERASED_DOCUMENT_DATA_DESCRIPTION: &DataDescription = &DataDescription {
    document_type: ERASED_DOCUMENT_TYPE,
    collection_of: Collection::None,
    fields: vec![],
};

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    pub version: u8,
    modules: Vec<DataDescription>,
}

impl DataSchema {
    #[must_use]
    pub fn new(version: u8, modules: Vec<DataDescription>) -> Self {
        DataSchema { version, modules }
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

        let mut refs = Refs::default();

        for field in &data_description.fields {
            let value = if let Some(value) = data.get(field.name) {
                value
            } else {
                continue;
            };

            if collection_ref_fields.contains(&field.name) {
                refs.collections.extend(field.extract_refs(value));
            } else {
                refs.documents.extend(field.extract_refs(value));
            }

            refs.blobs.extend(field.extract_blob_ids(value));
        }

        Ok(refs)
    }

    pub fn get_data_description(&self, document_type: impl AsRef<str>) -> Result<&DataDescription> {
        let document_type = document_type.as_ref();

        if document_type == ERASED_DOCUMENT_TYPE {
            return Ok(ERASED_DOCUMENT_DATA_DESCRIPTION);
        }

        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or_else(|| anyhow!("Unknown document type: {}", document_type))
    }

    pub fn get_title(&self, document: &Document) -> Result<String> {
        let data_description = self.get_data_description(&document.document_type)?;

        let title_field = if let Some(title_field) = data_description.pick_title_field() {
            title_field
        } else {
            return Ok(format!("{} {}", document.document_type, document.id));
        };

        document
            .data
            .get_str(title_field.name)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("title field {} is missing", title_field.name))
    }

    #[must_use]
    pub fn get_document_types(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.document_type)
            .collect()
    }
}
