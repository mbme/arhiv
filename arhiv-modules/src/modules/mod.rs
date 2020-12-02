use anyhow::*;
use serde_json::Map;
use std::collections::HashMap;
use std::collections::HashSet;
mod data_description;

use arhiv::entities::{Document, Id};
pub use data_description::*;
use serde_json::Value;

use crate::generator::Generator;
use crate::markup::MarkupString;

pub type DocumentData = Map<String, Value>;

pub struct DocumentDataManager {
    pub modules: HashMap<String, DataDescription>,
}

impl DocumentDataManager {
    pub fn new() -> DocumentDataManager {
        DocumentDataManager {
            modules: get_modules(),
        }
    }

    pub fn create(&self, document_type: String) -> Result<DocumentData> {
        self.create_with_data(document_type, Map::new())
    }

    pub fn create_with_data(
        &self,
        document_type: String,
        initial_values: DocumentData,
    ) -> Result<DocumentData> {
        let description = self
            .modules
            .get(&document_type)
            .ok_or(anyhow!("Unknown document type {}", &document_type))?;

        let mut result: DocumentData = Map::new();
        result.insert("type".to_string(), Value::String(document_type));

        for (name, field) in description.fields.iter() {
            if let Some(value) = initial_values.get(name) {
                result.insert(name.clone(), (*value).clone());
                continue;
            }

            match &field.field_type {
                FieldType::String | FieldType::MarkupString => {
                    result.insert(name.clone(), Value::from(""));
                    break;
                }
                FieldType::Enum(values) => {
                    let value = values.get(0).expect("enum must contain values");
                    result.insert(name.clone(), Value::String((*value).clone()));
                    break;
                }
                FieldType::Ref => {
                    bail!("initial value for Ref must be provided");
                }
            }
        }

        Ok(result)
    }

    fn get_data_description(&self, data: &DocumentData) -> Result<&DataDescription> {
        let document_type = data
            .get("type")
            .ok_or(anyhow!("document data must have type"))?;
        let document_type = document_type
            .as_str()
            .ok_or(anyhow!("document type must be string"))?;

        self.modules
            .get(document_type)
            .ok_or(anyhow!("Unknown document type {}", &document_type))
    }

    fn extract_refs(&self, data: &DocumentData) -> Result<HashSet<Id>> {
        let mut result = HashSet::new();

        let data_description = self.get_data_description(data)?;

        for (name, field) in data_description.fields.iter() {
            match field.field_type {
                FieldType::MarkupString => {
                    let value: MarkupString = serde_json::from_value(
                        data.get(name)
                            .expect(&format!("field '{}' must be present", name))
                            .clone(),
                    )
                    .expect("field must parse");

                    result.extend(value.extract_refs());
                }
                FieldType::Ref => {
                    let value: Id = serde_json::from_value(
                        data.get(name).expect("field must be present").clone(),
                    )
                    .expect("field must parse");

                    result.insert(value);
                }
                FieldType::String | FieldType::Enum(_) => {
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
        let refs = self.extract_refs(&data)?;
        document.refs = refs;

        Ok(())
    }

    pub fn gen_data(&self, data: &mut DocumentData, generator: &Generator) -> Result<()> {
        let description = self.get_data_description(data)?;

        for (name, field) in description.fields.iter() {
            match field.field_type {
                FieldType::String => {
                    data.insert(name.clone(), generator.gen_string().into());
                }
                FieldType::MarkupString => {
                    data.insert(name.clone(), generator.gen_markup_string(1, 8).0.into());
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn get_modules() -> HashMap<String, DataDescription> {
    let mut modules: HashMap<String, DataDescription> = HashMap::new();

    {
        let module = include_str!("./note.json");
        let module: DataDescription = serde_json::from_str(module).expect("module must be valid");
        modules.insert(module.document_type.clone(), module);
    }
    {
        let module = include_str!("./project.json");
        let module: DataDescription = serde_json::from_str(module).expect("module must be valid");
        modules.insert(module.document_type.clone(), module);
    }
    {
        let module = include_str!("./task.json");
        let module: DataDescription = serde_json::from_str(module).expect("module must be valid");
        modules.insert(module.document_type.clone(), module);
    }

    // FIXME deny "type" field
    // FIXME validate data description
    // FIXME generate json schema based on data description

    modules
}
