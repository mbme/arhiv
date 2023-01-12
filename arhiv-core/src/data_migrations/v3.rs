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
        if document.document_type == "book collection" {
            let document = document.to_mut();
            document.data.set("books", json!([]));
        }
        if document.document_type == "contact collection" {
            let document = document.to_mut();
            document.data.set("contacts", json!([]));
        }
        if document.document_type == "film collection" {
            let document = document.to_mut();
            document.data.set("films", json!([]));
        }
        if document.document_type == "game collection" {
            let document = document.to_mut();
            document.data.set("games", json!([]));
        }
        if document.document_type == "project" {
            let document = document.to_mut();
            document.data.set("tasks", json!([]));
        }
        if document.document_type == "track collection" {
            let document = document.to_mut();
            document.data.set("tracks", json!([]));
        }

        Ok(())
    }
}
