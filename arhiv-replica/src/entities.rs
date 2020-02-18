use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    rev: u32,
    #[serde(rename = "type")]
    document_type: String,
    schema_version: u8,
    created_at: String,
    updated_at: String,
    refs: Vec<String>,
    attachment_refs: Vec<String>,
    deleted: bool,
    props: HashMap<String, Value>,
}

impl Document {
    pub fn parse(src: &str) -> Result<Document> {
        serde_json::from_str(src).context("Failed to parse document json")
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }
}

impl std::str::FromStr for Document {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Document> {
        Document::parse(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    rev: u32,
    created_at: String,
}

impl Attachment {
    pub fn parse(src: &str) -> Result<Attachment> {
        serde_json::from_str(src).context("Failed to parse attachment json")
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize attachment to json")
    }
}

impl std::str::FromStr for Attachment {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Attachment> {
        Attachment::parse(data)
    }
}
