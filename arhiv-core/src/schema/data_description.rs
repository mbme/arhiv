use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::entities::{Document, Id};
use crate::markup::MarkupStr;

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    pub modules: Vec<DataDescription>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataDescription {
    pub document_type: &'static str,
    pub is_internal: bool,
    pub collection_of: Option<Collection>,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub optional: bool,
}

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},
    NaturalNumber {},
    MarkupString {},
    Ref(&'static str),
    Enum(Vec<&'static str>),
    ISBN {},
    Date {},
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub item_type: &'static str,
}

pub type DocumentData = Map<String, Value>;

impl DataDescription {
    pub fn create(&self, initial_values: DocumentData) -> Result<DocumentData> {
        let mut result: DocumentData = Map::new();

        for field in &self.fields {
            if let Some(value) = initial_values.get(field.name) {
                result.insert(field.name.to_string(), (*value).clone());
            }
        }

        Ok(result)
    }

    fn extract_refs(&self, data: &DocumentData) -> Result<HashSet<Id>> {
        let mut result = HashSet::new();

        for field in &self.fields {
            let value = {
                match (field.optional, data.get(field.name)) {
                    (true, None) => {
                        continue;
                    }
                    (false, None) => {
                        bail!("field {} must be present", field.name);
                    }
                    (_, Some(value)) => value,
                }
            };

            match field.field_type {
                FieldType::MarkupString {} => {
                    let markup: MarkupStr = value.as_str().expect("field must be string").into();

                    result.extend(markup.extract_refs());
                }
                FieldType::Ref(_) => {
                    // FIXME check ref document type
                    let value: Id =
                        serde_json::from_value(value.clone()).expect("field must parse");

                    result.insert(value);
                }
                _ => {
                    continue;
                }
            }
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

impl DataSchema {
    pub fn get_data_description(&self, document_type: impl AsRef<str>) -> Result<&DataDescription> {
        let document_type = document_type.as_ref();

        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or(anyhow!("Unknown document type: {}", document_type))
    }

    pub fn update_refs(&self, document: &mut Document) -> Result<()> {
        let data = {
            match &document.data {
                Value::Object(data) => data,
                _ => {
                    bail!("Document data must be an object");
                }
            }
        };

        document.refs = self
            .get_data_description(&document.document_type)?
            .extract_refs(&data)?;

        Ok(())
    }

    pub fn get_collection_type(&self, document_type: &str) -> Option<&'static str> {
        self.modules
            .iter()
            .find_map(|module| match module.collection_of {
                Some(ref collection_of) if collection_of.item_type == document_type => {
                    Some(module.document_type)
                }
                _ => None,
            })
    }

    pub fn get_title<'doc>(&self, document: &'doc Document) -> Result<&'doc str> {
        let data_description = self.get_data_description(&document.document_type)?;

        let title_field = data_description.pick_title_field()?;

        document
            .get_field_str(title_field.name)
            .ok_or(anyhow!("title field missing"))
    }
}

impl Field {
    pub fn get_enum_values(&self) -> Result<&Vec<&'static str>> {
        match self.field_type {
            FieldType::Enum(ref values) => Ok(values),
            _ => bail!("field {} isn't enum", self.name),
        }
    }
}
