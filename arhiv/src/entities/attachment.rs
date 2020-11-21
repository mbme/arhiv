use super::AttachmentSource;
use super::{Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use rs_utils::ensure_file_exists;
use rs_utils::get_file_hash_sha256;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Id,
    pub rev: Revision,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub filename: String,
    pub archived: bool, // data has been removed
}

impl Attachment {
    pub(crate) fn from(source: &AttachmentSource) -> Result<Attachment> {
        ensure_file_exists(&source.file_path)?;

        Ok(Attachment {
            id: source.id.clone(),
            rev: Revision::STAGING,
            hash: get_file_hash_sha256(&source.file_path)?,
            created_at: Utc::now(),
            filename: source.filename.clone(),
            archived: false,
        })
    }

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

impl fmt::Display for Attachment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Attachment {} \"{}\"]", self.id, self.filename)
    }
}
