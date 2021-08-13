use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use super::{field::*, search::MultiSearch};
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

    pub fn search(&self, data: &Value, pattern: &str) -> Result<u32> {
        let title_field = self.pick_title_field()?;

        let mut final_score = 0;
        let multi_search = MultiSearch::new(pattern);

        for field in &self.fields {
            let value = if let Some(value) = data.get(field.name) {
                value
            } else {
                continue;
            };

            let search_data = if let Some(search_data) = field.extract_search_data(value)? {
                search_data
            } else {
                continue;
            };

            let mut score = multi_search.search(&search_data);

            if field.name == title_field.name {
                score *= 3;
            }

            final_score += score;
        }

        Ok(final_score)
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
