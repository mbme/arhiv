use anyhow::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use uuid::Uuid;

pub type Revision = u32;
pub type Id = String;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Id,
    pub rev: Revision,
    pub created_at: DateTime<Utc>,
    pub filename: String,
}

impl Attachment {
    pub fn new(filename: &str) -> Attachment {
        Attachment {
            id: gen_id(),
            rev: 0,
            created_at: Utc::now(),
            filename: filename.to_owned(),
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize attachment to json")
    }

    pub fn is_staged(&self) -> bool {
        self.rev == 0
    }
}

impl std::str::FromStr for Attachment {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Attachment> {
        serde_json::from_str(data).context("Failed to parse attachment json")
    }
}

impl fmt::Display for Attachment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Attachment {} \"{}\"]", self.id, self.filename)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    pub base_rev: Revision,
    pub documents: Vec<Document>,
    pub attachments: Vec<Attachment>,
}

impl Changeset {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize Changeset to json")
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty() && self.attachments.is_empty()
    }
}

impl fmt::Display for Changeset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Changeset: {} documents, {} attachments since {}]",
            self.documents.len(),
            self.attachments.len(),
            self.base_rev,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetResponse {
    // replica storage revision
    pub base_rev: Revision,

    // primary storage revision
    pub latest_rev: Revision,

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

fn gen_id() -> Id {
    Uuid::new_v4().to_hyphenated().to_string()
}
