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
        if document.document_type.is("book") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type.is("contact") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type.is("film") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type.is("game") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type.is("game") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        if document.document_type.is("task") {
            let document = document.to_mut();
            document.data.remove("project");
        }

        if document.document_type.is("track") {
            let document = document.to_mut();
            document.data.remove("collections");
        }

        Ok(())
    }
}
