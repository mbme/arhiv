use anyhow::*;
use serde::Serialize;

use super::data_description::*;
use crate::entities::Document;

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    pub modules: Vec<DataDescription>,
}

impl DataSchema {
    pub fn get_data_description(&self, document_type: impl AsRef<str>) -> Result<&DataDescription> {
        let document_type = document_type.as_ref();

        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or(anyhow!("Unknown document type: {}", document_type))
    }

    pub(crate) fn update_refs(&self, document: &mut Document) -> Result<()> {
        document.refs = self
            .get_data_description(&document.document_type)?
            .extract_refs(&document.data)?;

        Ok(())
    }

    pub fn get_title<'doc>(&self, document: &'doc Document) -> Result<&'doc str> {
        let data_description = self.get_data_description(&document.document_type)?;

        let title_field = data_description.pick_title_field()?;

        document
            .data
            .get_str(title_field.name)
            .ok_or(anyhow!("title field {} is missing", title_field.name))
    }
}
