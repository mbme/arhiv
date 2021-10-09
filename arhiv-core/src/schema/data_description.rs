use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use super::{field::*, search::MultiSearch};

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
    pub fn pick_title_field(&self) -> Option<&Field> {
        self.fields.iter().find(|field| {
            matches!(
                field.field_type,
                FieldType::String {} | FieldType::MarkupString {}
            )
        })
    }

    pub fn search(&self, data: &Value, pattern: &str) -> Result<usize> {
        let title_field = self.pick_title_field();

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

            // increase score if field is a title
            if title_field.map_or(false, |title_field| title_field.name == field.name) {
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
            .ok_or_else(|| {
                anyhow!(
                    "can't find field {} in document type {}",
                    name,
                    self.document_type
                )
            })
    }
}
