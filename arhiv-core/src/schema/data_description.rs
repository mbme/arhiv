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

impl DataSchema {
    pub fn get_data_description_by_type(&self, document_type: &str) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or(anyhow!("Unknown document type {}", document_type))
    }

    pub fn create(&self, document_type: impl Into<String>) -> Result<DocumentData> {
        self.create_with_initial_values(document_type.into(), Map::new())
    }

    pub fn create_with_initial_values(
        &self,
        document_type: impl AsRef<str>,
        initial_values: DocumentData,
    ) -> Result<DocumentData> {
        let description = self.get_data_description_by_type(document_type.as_ref())?;

        let mut result: DocumentData = Map::new();

        for field in &description.fields {
            if let Some(value) = initial_values.get(field.name) {
                result.insert(field.name.to_string(), (*value).clone());
                continue;
            }

            if field.optional {
                continue;
            }

            match &field.field_type {
                FieldType::String {} | FieldType::MarkupString {} => {
                    result.insert(field.name.to_string(), Value::from(""));
                }
                FieldType::Enum(values) => {
                    let value = values.get(0).expect("enum must contain values");
                    result.insert(field.name.to_string(), Value::String(value.to_string()));
                }
                _ => {
                    bail!("initial value for {:?} must be provided", field);
                }
            }
        }

        Ok(result)
    }

    fn extract_refs(&self, document_type: &str, data: &DocumentData) -> Result<HashSet<Id>> {
        let mut result = HashSet::new();

        let data_description = self.get_data_description_by_type(document_type)?;

        for field in &data_description.fields {
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

    pub fn update_refs(&self, document: &mut Document) -> Result<()> {
        let data = {
            match &document.data {
                Value::Object(data) => data,
                _ => {
                    bail!("Document data must be an object");
                }
            }
        };
        let refs = self.extract_refs(&document.document_type, &data)?;
        document.refs = refs;

        Ok(())
    }

    pub fn pick_title_field(&self, document_type: &str) -> Result<&Field> {
        let description = self.get_data_description_by_type(document_type)?;

        description
            .fields
            .iter()
            .find(|field| match field.field_type {
                FieldType::String {} | FieldType::MarkupString {} => true,
                _ => false,
            })
            .ok_or(anyhow!("Failed to pick title field for {}", document_type))
    }

    pub fn extract_search_data(&self, document_type: &str, data: &str) -> Result<String> {
        let field = {
            if let Ok(field) = self.pick_title_field(document_type) {
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
}

impl DataDescription {
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

impl Field {
    pub fn get_enum_values(&self) -> Result<&Vec<&'static str>> {
        match self.field_type {
            FieldType::Enum(ref values) => Ok(values),
            _ => bail!("field {} isn't enum", self.name),
        }
    }
}
