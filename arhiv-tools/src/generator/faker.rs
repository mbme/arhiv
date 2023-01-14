use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use rand::{prelude::*, thread_rng};

use arhiv_core::{Arhiv, BazaConnectionExt};
use baza::{
    entities::{Document, DocumentData, Id},
    schema::FieldType,
    BazaConnection,
};
use rs_utils::{is_image_filename, workspace_relpath};

use super::TextGenerator;

fn list_attachments() -> Vec<String> {
    let mut attachments: Vec<String> = vec![];

    let dir = workspace_relpath("resources");
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

        let mut tx = arhiv.baza.get_tx()?;
        for file_path in list_attachments() {
            let attachment = tx.create_attachment(&file_path, false)?;

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

    fn create_fake(&self, tx: &BazaConnection, document_type: &str, subtype: &str) -> Document {
        let description = self
            .arhiv
            .baza
            .get_schema()
            .get_data_description(document_type)
            .unwrap();

        let mut data = DocumentData::new();
        let mut rng = thread_rng();
        for field in description.iter_fields(subtype) {
            match &field.field_type {
                FieldType::String {} => {
                    let (min, max) = self
                        .get_field_size_limits(document_type, field.name)
                        .unwrap_or((1, 8));
                    data.set(field.name, self.generator.gen_string(min, max));
                }
                FieldType::MarkupString {} => {
                    let (min, max) = self
                        .get_field_size_limits(document_type, field.name)
                        .unwrap_or((1, 8));

                    let title = self.generator.gen_string(1, 5);
                    let markup = self.generator.gen_markup_string(min, max);

                    data.set(field.name, format!("# {}\n{}", title, markup));
                }
                FieldType::Enum(values) => {
                    let value: &str = values.choose(&mut rng).unwrap();
                    data.set(field.name, value);
                }
                FieldType::RefList(child_document_type) => {
                    let child_quantity = self.get_quantity_limit(child_document_type);

                    let mut child_refs = Vec::new();
                    for _ in 0..child_quantity {
                        let mut child_document = self.create_fake(tx, child_document_type, "");

                        tx.stage_document(&mut child_document)
                            .expect("must be able to save child document");

                        child_refs.push(child_document.id);
                    }

                    data.set(field.name, child_refs);

                    println!("Generated {child_quantity} child {child_document_type} for {document_type}",);
                }
                _ => {}
            }
        }

        let mut document = Document::new_with_data(document_type, subtype, data);

        tx.stage_document(&mut document)
            .expect("must be able to save document");

        document
    }

    pub fn create_fakes<S: Into<String>>(&self, document_type: S, subtype: &str) {
        let document_type = document_type.into();

        let quantity = self.get_quantity_limit(&document_type);

        let tx = self.arhiv.baza.get_tx().expect("must open transaction");

        for _ in 0..quantity {
            self.create_fake(&tx, &document_type, subtype);
        }

        tx.commit().expect("must commit");

        println!("Generated {} {}", quantity, document_type);
    }
}
