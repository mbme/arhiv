use super::{gen_id, Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Id,
    pub rev: Revision,
    #[serde(rename = "type")]
    pub document_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub refs: Vec<Id>,
    pub attachment_refs: Vec<Id>,
    pub archived: bool,
    pub data: Value,
}

impl Document {
    pub fn new(document_type: &str) -> Document {
        let now = Utc::now();

        Document {
            id: gen_id(),
            rev: 0,
            document_type: document_type.to_string(),
            created_at: now,
            updated_at: now,
            refs: vec![],
            attachment_refs: vec![],
            archived: false,
            data: Value::Object(Map::new()),
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }

    pub fn is_staged(&self) -> bool {
        self.rev == 0
    }

    pub fn is_committed(&self) -> bool {
        self.rev > 0
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
            "[Document {}:{} {}]",
            self.document_type, self.id, self.rev,
        )
    }
}
