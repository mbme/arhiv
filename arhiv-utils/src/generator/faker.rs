use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use rand::{prelude::*, thread_rng};

use arhiv_core::{
    definitions::Attachment,
    entities::*,
    schema::{Collection, FieldType},
    Arhiv,
};
use rs_utils::{is_image_filename, project_relpath};

use super::TextGenerator;

fn list_attachments() -> Vec<String> {
    let mut attachments: Vec<String> = vec![];

    let dir = project_relpath("../resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if is_image_filename(path) {
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

        let mut tx = arhiv.get_tx()?;
        for file_path in list_attachments() {
            let attachment = Attachment::create_tx(&file_path, false, arhiv, &mut tx)?;

            attachment_ids.push(attachment.id.clone());
        }
        tx.commit()?;

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

    fn create_fake(&self, document_type: String, mut data: DocumentData) -> Document {
        let description = self
            .arhiv
            .get_schema()
            .get_data_description(&document_type)
            .unwrap();

        let mut rng = thread_rng();
        for field in &description.fields {
            match &field.field_type {
                FieldType::String {} => {
                    let (min, max) = self
                        .get_field_size_limits(&document_type, field.name)
                        .unwrap_or((1, 8));
                    data.set(field.name, self.generator.gen_string(min, max));
                }
                FieldType::MarkupString {} => {
                    let (min, max) = self
                        .get_field_size_limits(&document_type, field.name)
                        .unwrap_or((1, 8));

                    let title = self.generator.gen_string(1, 5);
                    let markup = self.generator.gen_markup_string(min, max);

                    data.set(field.name, format!("# {}\n{}", title, markup));
                }
                FieldType::Enum(values) => {
                    let value: &str = values.choose(&mut rng).unwrap();
                    data.set(field.name, value);
                }
                _ => {}
            }
        }

        Document::new_with_data(document_type, data)
    }

    pub fn create_fakes<S: Into<String>>(&self, document_type: S) {
        let document_type = document_type.into();

        let data_description = self
            .arhiv
            .get_schema()
            .get_data_description(&document_type)
            .unwrap_or_else(|_| panic!("Unknown document_type {}", &document_type));

        let quantity = self.get_quantity_limit(&document_type);

        let mut tx = self.arhiv.get_tx().expect("must open transaction");

        let mut child_total: u32 = 0;
        for _ in 0..quantity {
            let mut document = self.create_fake(document_type.clone(), DocumentData::new());
            self.arhiv
                .tx_stage_document(&mut document, &mut tx)
                .expect("must be able to save document");

            if let Collection::Type {
                document_type: child_document_type,
                field,
            } = data_description.collection_of
            {
                let child_quantity = self.get_quantity_limit(child_document_type);

                for _ in 0..child_quantity {
                    let mut initial_values = DocumentData::new();
                    initial_values.set(field.to_string(), &document.id);

                    let mut child_document =
                        self.create_fake(child_document_type.to_string(), initial_values);

                    self.arhiv
                        .tx_stage_document(&mut child_document, &mut tx)
                        .expect("must be able to save child document");
                }

                child_total += child_quantity as u32;
            }
        }

        tx.commit().expect("must commit");

        if let Collection::Type {
            document_type: item_type,
            ..
        } = &data_description.collection_of
        {
            println!(
                "Generated {} {} and {} child {}",
                quantity, document_type, child_total, item_type
            );
        } else {
            println!("Generated {} {}", quantity, document_type);
        }
    }
}
