use anyhow::*;
use serde_json::Map;
use std::collections::HashSet;
mod data_description;

use crate::entities::{Document, Id};
pub use data_description::*;
use serde_json::Value;

use crate::markup::MarkupString;

pub type DocumentData = Map<String, Value>;

impl DataSchema {
    const SCHEMA: &'static str = include_str!("./schema.json");

    pub fn new() -> DataSchema {
        let schema: DataSchema =
            serde_json::from_str(DataSchema::SCHEMA).expect("schema must be valid");
        // FIXME validate data description
        // FIXME generate json schema based on data description
        // FIXME check if collection child type has field with name equal to collection type

        schema
    }

    pub fn get_data_description_by_type(&self, document_type: &str) -> Result<&DataDescription> {
        self.modules
            .iter()
            .find(|module| module.document_type == document_type)
            .ok_or(anyhow!("Unknown document type {}", document_type))
    }

    pub fn create(&self, document_type: String) -> Result<DocumentData> {
        self.create_with_data(document_type, Map::new())
    }

    pub fn create_with_data(
        &self,
        document_type: String,
        initial_values: DocumentData,
    ) -> Result<DocumentData> {
        let description = self.get_data_description_by_type(&document_type)?;

        let mut result: DocumentData = Map::new();

        for field in &description.fields {
            if let Some(value) = initial_values.get(&field.name) {
                result.insert(field.name.clone(), (*value).clone());
                continue;
            }

            match &field.field_type {
                FieldType::String {} | FieldType::MarkupString {} => {
                    result.insert(field.name.clone(), Value::from(""));
                }
                FieldType::Enum(values) => {
                    let value = values.get(0).expect("enum must contain values");
                    result.insert(field.name.clone(), Value::String((*value).clone()));
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
                        data.get(&field.name)
                            .expect(&format!("field '{}' must be present", field.name))
                            .clone(),
                    )
                    .expect("field must parse");

                    result.extend(value.extract_refs());
                }
                FieldType::Ref(_) => {
                    // FIXME check ref document type
                    let value: Id = serde_json::from_value(
                        data.get(&field.name)
                            .expect("field must be present")
                            .clone(),
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
}
