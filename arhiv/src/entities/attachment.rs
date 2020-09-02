use super::{Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use rs_utils::gen_uuid;
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
    pub(crate) fn new(hash: String, filename: &str) -> Attachment {
        Attachment {
            id: gen_uuid(),
            rev: 0,
            hash,
            created_at: Utc::now(),
            filename: filename.to_owned(),
            archived: false,
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize attachment to json")
    }

    pub fn is_staged(&self) -> bool {
        self.rev == 0
    }

    pub fn is_committed(&self) -> bool {
        self.rev > 0
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
