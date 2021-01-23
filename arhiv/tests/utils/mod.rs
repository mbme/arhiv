use anyhow::*;
use arhiv::{entities::Document, ListPage, TestArhiv};
use std::fs;

pub fn new_prime() -> TestArhiv {
    TestArhiv::new(true, 0)
}

pub fn new_replica() -> TestArhiv {
    TestArhiv::new(false, 0)
}

pub fn new_replica_with_port(port: u16) -> TestArhiv {
    TestArhiv::new(false, port)
}

pub fn empty_document() -> Document {
    Document::new("test_type".to_string(), serde_json::json!({}))
}

pub fn new_document(value: serde_json::Value) -> Document {
    Document::new("test_type".to_string(), value)
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}

pub fn get_values(page: ListPage<Document>) -> Vec<serde_json::Value> {
    page.items.into_iter().map(|item| item.data).collect()
}
