use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::entities::{Document, DocumentData, Refs, ERASED_DOCUMENT_TYPE};

pub use data_description::*;
pub use data_migration::*;
pub use field::*;

mod data_description;
mod data_migration;
mod field;
mod search;

const ERASED_DOCUMENT_DATA_DESCRIPTION: &DataDescription = &DataDescription {
    document_type: ERASED_DOCUMENT_TYPE,
    fields: vec![],
    subtypes: None,
};

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    modules: Vec<DataDescription>,
}

impl DataSchema {
    #[must_use]
    pub fn new(mut modules: Vec<DataDescription>) -> Self {
        modules.push(ERASED_DOCUMENT_DATA_DESCRIPTION.clone());

        DataSchema { modules }
    }

    pub fn extract_refs(
        &self,
        document_type: &str,
        subtype: &str,
        data: &DocumentData,
    ) -> Result<Refs> {
        let data_description = self.get_data_description(document_type)?;

        let mut refs = Refs::default();

        for field in data_description.iter_fields(subtype) {
            if let Some(value) = data.get(field.name) {
                refs.documents.extend(field.extract_refs(value));
                refs.collection.extend(field.extract_collection_refs(value));
                refs.blobs.extend(field.extract_blob_ids(value));
            }
        }

        Ok(refs)
    }

    pub fn get_data_description(&self, document_type: &str) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or_else(|| {
                let types = self
                    .modules
                    .iter()
                    .map(|module| module.document_type)
                    .collect::<Vec<_>>()
                    .join(", ");

                anyhow!(
                    "Unknown document type {}, must be one of [{}]",
                    document_type,
                    types
                )
            })
    }

    pub fn get_title(&self, document: &Document) -> Result<String> {
        let data_description = self.get_data_description(&document.document_type)?;

        let title_field =
            if let Some(title_field) = data_description.pick_title_field(&document.subtype) {
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
