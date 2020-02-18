use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub type Revision = u32;
pub type Id = String;
pub type Moment = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Id,
    rev: Revision,
    #[serde(rename = "type")]
    document_type: String,
    schema_version: u8,
    created_at: Moment,
    updated_at: Moment,
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
    pub id: Id,
    rev: Revision,
    created_at: Moment,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    pub replica_rev: Revision,
    pub documents: Vec<Document>,
    pub attachments: Vec<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangesetResponseStatus {
    Accepted,
    Outdated,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetResponse {
    status: ChangesetResponseStatus,

    // replica storage revision
    replica_rev: Revision,

    // primary storage revision
    primary_rev: Revision,

    // documents with rev > replica_rev
    documents: Vec<Document>,

    // attachments with rev > replica_rev
    attachments: Vec<Attachment>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_serialize_changeset_response_status() {
        assert_eq!(
            serde_json::to_string(&ChangesetResponseStatus::Outdated).unwrap(),
            r#""outdated""#
        );
    }

    #[test]
    fn it_can_parse_changeset_response_status() {
        assert_eq!(
            serde_json::from_str::<ChangesetResponseStatus>(r#""accepted""#).unwrap(),
            ChangesetResponseStatus::Accepted
        );
    }
}
