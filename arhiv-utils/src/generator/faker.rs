use anyhow::*;
use arhiv::entities::*;
use arhiv::Arhiv;
use rand::prelude::*;
use rand::thread_rng;
use rs_utils::project_relpath;
use serde_json::Map;
use std::{collections::HashMap, fs};

use arhiv::schema::{DocumentData, FieldType};

use super::TextGenerator;

fn list_attachments() -> Vec<String> {
    let mut attachments: Vec<String> = vec![];

    let dir = project_relpath("../resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            attachments.push(path.to_string());
        }
    }

    attachments
}

pub struct Faker<'a> {
    arhiv: &'a Arhiv,
    generator: TextGenerator,
    pub quantity_limits: HashMap<String, u32>,
    pub field_size_limits: HashMap<(String, String), (u32, u32)>,
}

impl<'a> Faker<'a> {
    pub fn new(arhiv: &'a Arhiv) -> Result<Faker> {
        let mut attachment_ids: Vec<Id> = vec![];
        for file_path in list_attachments() {
            let document = arhiv.add_attachment(file_path, true)?;
            attachment_ids.push(document.id);
        }

        let generator = TextGenerator::new(attachment_ids);

        Ok(Faker {
            arhiv,
            generator,
            quantity_limits: HashMap::new(),
            field_size_limits: HashMap::new(),
        })
    }

    fn get_quantity_limit(&self, document_type: &str) -> u32 {
        *self.quantity_limits.get(document_type).unwrap_or(&30)
    }

    fn get_field_size_limits(&self, document_type: &str, field_name: &str) -> Option<(u32, u32)> {
        self.field_size_limits
            .get(&(document_type.to_string(), field_name.to_string()))
            .map(|(min, max)| (*min, *max))
    }

    fn create_fake(&self, document_type: String, initial_values: DocumentData) -> Document {
        let mut data = self
            .arhiv
            .schema
            .create_with_initial_values(document_type.clone(), initial_values)
            .expect(&format!(
                "Failed to create data for document_type {}",
                document_type
            ));

        let description = self
            .arhiv
            .schema
            .get_data_description_by_type(&document_type)
            .unwrap();

        let mut rng = thread_rng();
        for field in &description.fields {
            match &field.field_type {
                FieldType::String {} => {
                    let (min, max) = self
                        .get_field_size_limits(&document_type, &field.name)
                        .unwrap_or((1, 8));
                    data.insert(
                        field.name.clone(),
                        self.generator.gen_string(min, max).into(),
                    );
                }
                FieldType::MarkupString {} => {
                    let (min, max) = self
                        .get_field_size_limits(&document_type, &field.name)
                        .unwrap_or((1, 8));
                    data.insert(
                        field.name.clone(),
                        self.generator.gen_markup_string(min, max).0.into(),
                    );
                }
                FieldType::Enum(values) => {
                    let value: &str = values.choose(&mut rng).unwrap();
                    data.insert(field.name.clone(), value.into());
                }
                _ => {}
            }
        }

        let mut document = Document::new(document_type, data.into());
        self.arhiv
            .schema
            .update_refs(&mut document)
            .expect("Failed to update refs");

        document
    }

    pub fn create_fakes<S: Into<String>>(&self, document_type: S) {
        let document_type = document_type.into();

        let data_description = self
            .arhiv
            .schema
            .get_data_description_by_type(&document_type)
            .expect(&format!("Unknown document_type {}", &document_type));

        let quantity = self.get_quantity_limit(&document_type);

        let mut child_total: u32 = 0;
        for _ in 0..quantity {
            let document = self.create_fake(document_type.clone(), Map::new());
            let id = document.id.clone();
            self.arhiv
                .stage_document(document)
                .expect("must be able to save document");

            if let Some(collection_options) = &data_description.collection_of {
                let child_document_type = collection_options.item_type.clone();
                let child_quantity = self.get_quantity_limit(&child_document_type);

                for _ in 0..child_quantity {
                    let mut initial_values = Map::new();
                    initial_values
                        .insert(document_type.clone(), serde_json::to_value(&id).unwrap());
                    let child_document =
                        self.create_fake(child_document_type.clone(), initial_values);

                    self.arhiv
                        .stage_document(child_document)
                        .expect("must be able to save child document");
                }

                child_total += child_quantity as u32;
            }
        }

        if let Some(collection_options) = &data_description.collection_of {
            println!(
                "Generated {} {} and {} child {}",
                quantity, document_type, child_total, collection_options.item_type
            );
        } else {
            println!("Generated {} {}", quantity, document_type);
        }
    }
}
