use std::fs;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use rs_utils::{generate_temp_path, SelfSignedCertificate};

use crate::{
    entities::{BLOBId, Document, DocumentType, Id, Revision, BLOB},
    schema::DataSchema,
    sync::Changeset,
    Baza, BazaOptions, Filter, ListPage,
};

mod attachment;
mod auto_commit_service;
mod crud;
mod locks;
mod query;
mod sync;
mod validation;

#[cfg(test)]
impl Baza {
    pub fn new_with_schema(schema: DataSchema) -> Self {
        let temp_dir = generate_temp_path("TestBaza", "");

        Baza::create(
            BazaOptions {
                root_dir: temp_dir,
                schema,
            },
            "test1234".into(),
        )
        .expect("must create baza")
    }

    pub fn new_test_baza() -> Self {
        Self::new_test_baza_with_id("0")
    }

    pub fn new_test_baza_with_id(id: &str) -> Self {
        let baza = Baza::new_with_schema(DataSchema::new_test_schema());

        let tx = baza.get_tx().unwrap();
        let id = id.try_into().unwrap();
        tx.set_instance_id(&id).unwrap();
        tx.commit().unwrap();

        baza
    }

    pub fn add_document(&self, id: Id, rev: Value) -> Result<Document> {
        let tx = self.get_tx()?;

        let document = new_document_snapshot(id, rev);
        tx.put_document(&document, None)?;
        tx.commit()?;

        Ok(document)
    }

    pub fn list_documents(&self, filter: impl AsRef<Filter>) -> Result<ListPage> {
        let conn = self.get_connection()?;

        conn.list_documents(filter.as_ref())
    }

    pub fn get_document(&self, id: impl Into<Id>) -> Result<Option<Document>> {
        let conn = self.get_connection()?;

        conn.get_document(&id.into())
    }

    pub fn get_blob(&self, id: &BLOBId) -> Result<BLOB> {
        let conn = self.get_connection()?;

        Ok(conn.get_blob(id))
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
    Document::new_with_data(DocumentType::new("test_type"), value.try_into().unwrap())
}

pub fn new_empty_document() -> Document {
    new_document(json!({}))
}

pub fn new_document_snapshot(id: impl Into<Id>, revision: Value) -> Document {
    let mut document = new_document(json!({}));
    document.id = id.into();

    document.rev = Revision::from_value(revision).expect("must be a valid revision");

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

pub fn get_values(page: ListPage) -> Vec<Value> {
    page.items
        .into_iter()
        .map(|item| item.data.into())
        .collect()
}

pub fn new_certificate() -> SelfSignedCertificate {
    SelfSignedCertificate::new_x509("test").unwrap()
}
