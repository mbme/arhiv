use super::{Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;

pub const DELETED_TYPE: &'static str = "deleted";

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Id,
    pub rev: Revision,
    pub document_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub refs: HashSet<Id>,
    pub archived: bool,
    pub data: Value,
}

impl Document {
    pub fn new(document_type: String, data: Value) -> Document {
        let now = Utc::now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            document_type,
            created_at: now,
            updated_at: now,
            refs: HashSet::new(),
            archived: false,
            data,
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }

    pub fn delete(&mut self) {
        self.document_type = DELETED_TYPE.to_string();
        self.rev = Revision::STAGING;
        self.refs = HashSet::new();
        self.archived = true;
        self.data = serde_json::json!({});
        self.updated_at = Utc::now();
    }

    pub fn is_deleted(&self) -> bool {
        self.document_type == DELETED_TYPE
    }
}

impl std::str::FromStr for Document {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Document> {
        serde_json::from_str(data).context("Failed to parse document json")
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Document {}/{} {}]",
            self.document_type, self.id, self.rev,
        )
    }
}
