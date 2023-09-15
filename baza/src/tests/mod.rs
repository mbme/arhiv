use std::fs;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use rs_utils::generate_temp_path;

use crate::{
    entities::{Document, DocumentClass, Id},
    schema::{DataDescription, DataSchema, Field, FieldType},
    sync::{Changeset, Revision},
    Baza, SETTING_INSTANCE_ID,
};

mod crud;
mod sync;
mod sync_service;
mod validation;

#[cfg(test)]
impl Baza {
    pub fn new_with_schema(schema: DataSchema) -> Self {
        let temp_dir = generate_temp_path("TestBaza", "");

        Baza::create(temp_dir, schema, vec![]).expect("must create baza")
    }

    pub fn new_test_baza() -> Self {
        Self::new_test_baza_with_id("0")
    }

    pub fn new_test_baza_with_id(id: &str) -> Self {
        let baza = Baza::new_with_schema(DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "test_type",
                fields: vec![
                    Field {
                        name: "test",
                        field_type: FieldType::String {},
                        mandatory: false,
                        readonly: false,
                        for_subtypes: None,
                    },
                    Field {
                        name: "blob",
                        field_type: FieldType::BLOBId {},
                        mandatory: false,
                        readonly: false,
                        for_subtypes: None,
                    },
                ],
                subtypes: None,
            }],
        ));

        let tx = baza.get_tx().unwrap();
        let id = id.try_into().unwrap();
        tx.kvs_const_set(SETTING_INSTANCE_ID, &id).unwrap();
        tx.commit().unwrap();

        baza
    }

    pub fn add_document(&self, id: Id, rev: Value) -> Result<Document> {
        let tx = self.get_tx()?;

        let document = new_document_snapshot(id, rev);
        tx.put_document(&document)?;
        tx.commit()?;

        Ok(document)
    }
}

// Remove temporary Baza in tests
#[cfg(test)]
impl Drop for Baza {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.get_path_manager().root_dir)
            .expect("must be able to remove baza");
    }
}

pub fn new_document(value: Value) -> Document {
    Document::new_with_data(
        DocumentClass::new("test_type", ""),
        value.try_into().unwrap(),
    )
}

pub fn new_document_snapshot(id: impl Into<Id>, revision: Value) -> Document {
    let mut document = new_document(json!({}));
    document.id = id.into();

    document.rev = Revision::try_from_value(revision).expect("must be a valid revision");

    document
}

pub fn create_changeset(documents: Vec<Document>) -> Changeset {
    Changeset {
        data_version: 0,
        documents,
    }
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src).context("failed to read src file")?
        == fs::read(dst).context("failed to read dst file")?)
}
