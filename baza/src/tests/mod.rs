use std::fs;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::entities::{Document, DocumentType, Id, Revision};

// mod sync; FIXME enable

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

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src).context("failed to read src file")?
        == fs::read(dst).context("failed to read dst file")?)
}
