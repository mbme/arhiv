use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::entities::{DocumentType, ERASED_DOCUMENT_TYPE};

pub use attachment::*;
pub use data_description::*;
pub use data_migration::*;
pub use field::*;

mod attachment;
mod data_description;
mod data_migration;
mod field;

const ERASED_DOCUMENT_DATA_DESCRIPTION: &DataDescription = &DataDescription {
    document_type: ERASED_DOCUMENT_TYPE,
    title_format: "Erased document",
    fields: vec![],
};

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    name: String,
    modules: Vec<DataDescription>,
}

impl DataSchema {
    #[must_use]
    pub fn new(name: impl Into<String>, mut modules: Vec<DataDescription>) -> Self {
        modules.push(ERASED_DOCUMENT_DATA_DESCRIPTION.clone());

        DataSchema {
            name: name.into(),
            modules,
        }
    }

    #[must_use]
    pub fn get_app_name(&self) -> &str {
        &self.name
    }

    pub fn get_data_description(&self, document_type: &DocumentType) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| document_type.is(module.document_type))
            .ok_or_else(|| {
                let types = self.get_document_types().join(", ");

                anyhow!(
                    "Unknown document type {}, must be one of [{}]",
                    document_type,
                    types
                )
            })
    }

    pub fn iter_fields(
        &self,
        document_type: &DocumentType,
    ) -> Result<impl Iterator<Item = &Field>> {
        let data_description = self.get_data_description(document_type)?;

        Ok(data_description.fields.iter())
    }

    #[must_use]
    pub fn get_document_types(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.document_type)
            .collect()
    }
}
