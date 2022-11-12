use anyhow::Result;
use serde::Serialize;

use crate::entities::DocumentData;

use super::{field::*, search::MultiSearch};

#[derive(Serialize, Debug, Clone)]
pub struct DataDescription {
    pub document_type: &'static str,
    pub subtypes: Option<&'static [&'static str]>,
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
    #[must_use]
    pub fn pick_title_field(&self, subtype: &str) -> Option<&Field> {
        self.iter_fields(subtype).find(|field| {
            matches!(
                field.field_type,
                FieldType::String {} | FieldType::MarkupString {}
            )
        })
    }

    pub fn search(&self, subtype: &str, data: &DocumentData, pattern: &str) -> Result<usize> {
        let title_field = self.pick_title_field(subtype);

        let mut final_score = 0;
        let multi_search = MultiSearch::new(pattern);

        for field in self.iter_fields(subtype) {
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

    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&Field> {
        let name = name.as_ref();

        self.fields.iter().find(|field| field.name == name)
    }

    #[must_use]
    pub fn is_editable(&self, subtype: &str) -> bool {
        self.iter_fields(subtype).any(|field| !field.readonly)
    }

    #[must_use]
    pub fn is_supported_subtype(&self, subtype: &str) -> bool {
        self.subtypes.unwrap_or(&[""]).contains(&subtype)
    }

    pub fn iter_fields(&self, subtype: &str) -> impl Iterator<Item = &Field> {
        let subtype = subtype.to_string();

        self.fields
            .iter()
            .filter(move |field| field.for_subtype(&subtype))
    }

    pub fn is_collection(&self) -> bool {
        !matches!(self.collection_of, Collection::None)
    }
}
