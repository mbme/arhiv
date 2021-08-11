use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use super::field::*;
use crate::entities::{DocumentData, Id};

#[derive(Serialize, Debug, Clone)]
pub struct DataDescription {
    pub document_type: &'static str,
    pub is_internal: bool,
    pub collection_of: Collection,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Debug, Clone)]
pub enum Collection {
    None,
    Type {
        document_type: &'static str,
        field: &'static str,
    },
}

impl DataDescription {
    pub fn extract_refs(&self, data: &DocumentData) -> Result<HashSet<Id>> {
        let mut result = HashSet::new();

        for field in &self.fields {
            let value = if let Some(value) = data.get(field.name) {
                value
            } else {
                continue;
            };

            result.extend(field.get_refs(value));
        }

        Ok(result)
    }

    pub fn pick_title_field(&self) -> Result<&Field> {
        self.fields
            .iter()
            .find(|field| match field.field_type {
                FieldType::String {} | FieldType::MarkupString {} => true,
                _ => false,
            })
            .ok_or(anyhow!(
                "Failed to pick title field for {}",
                self.document_type
            ))
    }

    pub fn extract_search_data(&self, data: &str) -> Result<String> {
        let field = {
            if let Ok(field) = self.pick_title_field() {
                field
            } else {
                // else use whole data prop for search index
                return Ok(data.to_string());
            }
        };

        let data: Value = serde_json::from_str(data)?;

        data[field.name]
            .as_str()
            .map(|value| value.to_lowercase())
            .ok_or(anyhow!("failed to extract field {}", field.name))
    }

    pub fn get_field(&self, name: impl AsRef<str>) -> Result<&Field> {
        let name = name.as_ref();
        self.fields
            .iter()
            .find(|field| field.name == name)
            .ok_or(anyhow!(
                "can't find field {} in document type {}",
                name,
                self.document_type
            ))
    }
}
