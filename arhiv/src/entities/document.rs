use super::{AttachmentSource, Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use rs_utils::{ensure_file_exists, get_file_hash_sha256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;

pub const ATTACHMENT_TYPE: &'static str = "attachment";

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

    pub fn is_attachment(&self) -> bool {
        self.document_type == ATTACHMENT_TYPE
    }

    pub(crate) fn from(source: &AttachmentSource) -> Result<Document> {
        ensure_file_exists(&source.file_path)?;

        let info = AttachmentInfo {
            hash: get_file_hash_sha256(&source.file_path)?,
            filename: source.filename.clone(),
        };

        Ok(Document {
            id: source.id.clone(),
            ..Document::new(ATTACHMENT_TYPE.to_string(), serde_json::to_value(&info)?)
        })
    }

    pub fn get_attachment_info(self) -> Result<AttachmentInfo> {
        if !self.is_attachment() {
            bail!("Document {} isn't an attachment", self.id);
        }

        let info: AttachmentInfo = serde_json::from_value(self.data)?;

        Ok(info)
    }

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

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Document {} {}]", self.id, self.rev,)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachmentInfo {
    pub hash: String,
    pub filename: String,
}
