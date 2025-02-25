use serde_json::{json, Value};

use crate::entities::{Document, DocumentType};

pub fn new_document(value: Value) -> Document {
    Document::new_with_data(DocumentType::new("test_type"), value.try_into().unwrap())
}

pub fn new_empty_document() -> Document {
    new_document(json!({}))
}
