use arhiv::entities::*;
use arhiv::{entities::AttachmentSource, Arhiv};
use rand::prelude::*;
use rand::thread_rng;
use rs_utils::project_relpath;
use serde_json::Map;
use std::{collections::HashMap, fs};

use crate::modules::{DocumentData, DocumentDataManager, FieldType};

use super::TextGenerator;

fn create_attachments() -> Vec<AttachmentSource> {
    let mut attachments: Vec<AttachmentSource> = vec![];

    let dir = project_relpath("../resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            let attachment = AttachmentSource::new(path);
            attachments.push(attachment);
        }
    }

    attachments
}

pub struct Faker {
    attachments: Vec<AttachmentSource>,
    generator: TextGenerator,
    data_manager: DocumentDataManager,
    pub quantity_limits: HashMap<String, u32>,
    pub field_size_limits: HashMap<(String, String), (u32, u32)>,
}

impl Faker {
    pub fn new() -> Self {
        let attachments = create_attachments();
        let generator = TextGenerator::new(&attachments);
        let data_manager = DocumentDataManager::new();

        Faker {
            attachments,
            generator,
            data_manager,
            quantity_limits: HashMap::new(),
            field_size_limits: HashMap::new(),
        }
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
            .data_manager
            .create_with_data(document_type.clone(), initial_values)
            .expect(&format!(
                "Failed to create data for document_type {}",
                document_type
            ));

        let description = self
            .data_manager
            .get_data_description_by_type(&document_type)
            .unwrap();

        let mut rng = thread_rng();
        for field in &description.fields {
            match &field.field_type {
                FieldType::String => {
                    let (min, max) = self
                        .get_field_size_limits(&document_type, &field.name)
                        .unwrap_or((1, 8));
                    data.insert(
                        field.name.clone(),
                        self.generator.gen_string(min, max).into(),
                    );
                }
                FieldType::MarkupString => {
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

        let mut document = Document::new(data.into());
        self.data_manager
            .update_refs(&mut document)
            .expect("Failed to update refs");

        document
    }

    pub fn create_fakes<S: Into<String>>(&self, document_type: S, arhiv: &Arhiv) {
        let document_type = document_type.into();

        let data_description = self
            .data_manager
            .get_data_description_by_type(&document_type)
            .expect(&format!("Unknown document_type {}", &document_type));

        let quantity = self.get_quantity_limit(&document_type);

        let mut child_total: u32 = 0;
        for _ in 0..quantity {
            let document = self.create_fake(document_type.clone(), Map::new());
            let id = document.id.clone();
            arhiv
                .stage_document(document, self.attachments.clone())
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

                    arhiv
                        .stage_document(child_document, self.attachments.clone())
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
