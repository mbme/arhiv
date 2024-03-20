use std::borrow::Cow;

use anyhow::Result;
use serde_json::json;

use baza::{entities::Document, schema::DataMigration, BazaConnection};

pub struct DataSchema3;

/// Add new RefList fields to collection types
impl DataMigration for DataSchema3 {
    fn get_version(&self) -> u8 {
        3
    }

    fn update(&self, document: &mut Cow<Document>, _conn: &BazaConnection) -> Result<()> {
        if document.document_type.is("book collection") {
            let document = document.to_mut();
            document.data.set("books", json!([]));
        }
        if document.document_type.is("contact collection") {
            let document = document.to_mut();
            document.data.set("contacts", json!([]));
        }
        if document.document_type.is("film collection") {
            let document = document.to_mut();
            document.data.set("films", json!([]));
        }
        if document.document_type.is("game collection") {
            let document = document.to_mut();
            document.data.set("games", json!([]));
        }
        if document.document_type.is("project") {
            let document = document.to_mut();
            document.data.set("tasks", json!([]));
        }
        if document.document_type.is("track collection") {
            let document = document.to_mut();
            document.data.set("tracks", json!([]));
        }

        Ok(())
    }
}
