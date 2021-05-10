use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::entities::{Document, Id};
use crate::markup::MarkupString;

#[derive(Serialize, Debug, Clone)]
pub struct DataSchema {
    pub modules: Vec<DataDescription>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataDescription {
    pub document_type: &'static str,
    pub collection_of: Option<Collection>,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
}

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},
    MarkupString {},
    Ref(&'static str),
    Enum(Vec<&'static str>),
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

    pub fn create(&self, document_type: String) -> Result<DocumentData> {
        self.create_with_initial_values(document_type, Map::new())
    }

    pub fn create_with_initial_values(
        &self,
        document_type: String,
        initial_values: DocumentData,
    ) -> Result<DocumentData> {
        let description = self.get_data_description_by_type(&document_type)?;

        let mut result: DocumentData = Map::new();

        for field in &description.fields {
            if let Some(value) = initial_values.get(field.name) {
                result.insert(field.name.to_string(), (*value).clone());
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
                FieldType::Ref(_) => {
                    // FIXME check ref document type
                    bail!("initial value for Ref must be provided");
                }
            }
        }

        Ok(result)
    }

    fn extract_refs(&self, document_type: &str, data: &DocumentData) -> Result<HashSet<Id>> {
        let mut result = HashSet::new();

        let data_description = self.get_data_description_by_type(document_type)?;

        for field in &data_description.fields {
            match field.field_type {
                FieldType::MarkupString {} => {
                    let value: MarkupString = serde_json::from_value(
                        data.get(field.name)
                            .expect(&format!("field '{}' must be present", field.name))
                            .clone(),
                    )
                    .expect("field must parse");

                    result.extend(value.extract_refs());
                }
                FieldType::Ref(_) => {
                    // FIXME check ref document type
                    let value: Id = serde_json::from_value(
                        data.get(field.name).expect("field must be present").clone(),
                    )
                    .expect("field must parse");

                    result.insert(value);
                }
                FieldType::String {} | FieldType::Enum(_) => {
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

    pub fn get_field(&self, document: &Document, field: &str) -> Result<Value> {
        let value = document
            .data
            .get(field)
            .ok_or(anyhow!("can't find field {}", field))?;

        Ok(value.clone())
    }

    pub fn get_field_string(&self, document: &Document, field: &str) -> Result<String> {
        let value = self.get_field(document, field)?;

        serde_json::from_value(value).context("can't use value as String")
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
}
