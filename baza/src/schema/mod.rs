use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::entities::{DocumentClass, ERASED_DOCUMENT_TYPE};

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
    subtypes: None,
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
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_data_description(&self, document_type: &DocumentClass) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| module.document_type == document_type.document_type)
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
        document_type: &DocumentClass,
    ) -> Result<impl Iterator<Item = &Field>> {
        let subtype = document_type.subtype.clone();

        let data_description = self.get_data_description(document_type)?;

        let iter = data_description
            .fields
            .iter()
            .filter(move |field| field.for_subtype(&subtype));

        Ok(iter)
    }

    #[must_use]
    pub fn get_document_types(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|module| module.document_type)
            .collect()
    }
}
