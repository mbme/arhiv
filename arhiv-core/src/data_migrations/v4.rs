use std::borrow::Cow;

use anyhow::Result;

use baza::{entities::Document, schema::DataMigration, BazaConnection};

pub struct DataSchema4;

/// Remove old collection fields
impl DataMigration for DataSchema4 {
    fn get_version(&self) -> u8 {
        4
    }

    fn update(&self, document: &mut Cow<Document>, _conn: &BazaConnection) -> Result<()> {
        if document.document_type == "book" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type == "contact" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type == "film" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type == "game" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type == "game" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type == "task" {
            let document = document.to_mut();
            document.data.remove("project");
        }

        if document.document_type == "track" {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        Ok(())
    }
}
