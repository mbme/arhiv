use std::fs;

use anyhow::Result;
use serde_json::Value;

use crate::{entities::Document, ListPage};

pub fn empty_document() -> Document {
    Document::new("test_type", "")
}

pub fn new_document(value: Value) -> Document {
    Document::new_with_data("test_type", "", value.try_into().unwrap())
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}

pub fn get_values(page: ListPage) -> Vec<Value> {
    page.items
        .into_iter()
        .map(|item| item.data.into())
        .collect()
}
