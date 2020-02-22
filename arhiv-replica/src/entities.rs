use anyhow::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub type Revision = u32;
pub type Id = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Id,
    rev: Revision,
    #[serde(rename = "type")]
    document_type: String,
    schema_version: u8,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    refs: Vec<Id>,
    attachment_refs: Vec<Id>,
    deleted: bool,
    props: HashMap<String, Value>,
}

impl Document {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }
}

impl std::str::FromStr for Document {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Document> {
        serde_json::from_str(data).context("Failed to parse document json")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Id,
    rev: Revision,
    created_at: DateTime<Utc>,
}

impl Attachment {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize attachment to json")
    }
}

impl std::str::FromStr for Attachment {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Attachment> {
        serde_json::from_str(data).context("Failed to parse attachment json")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    pub replica_rev: Revision,
    pub documents: Vec<Document>,
    pub attachments: Vec<Attachment>,
}

impl Changeset {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize Changeset to json")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetResponse {
    // replica storage revision
    pub replica_rev: Revision,

    // primary storage revision
    pub primary_rev: Revision,

    // documents with rev > replica_rev
    pub documents: Vec<Document>,

    // attachments with rev > replica_rev
    pub attachments: Vec<Attachment>,
}

impl std::str::FromStr for ChangesetResponse {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<ChangesetResponse> {
        serde_json::from_str(data).context("Failed to parse ChangesetResponse")
    }
}
