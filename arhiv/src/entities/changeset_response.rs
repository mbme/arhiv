use anyhow::*;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Document, Revision};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetResponse {
    pub arhiv_id: String,

    // replica revision
    pub base_rev: Revision,

    // primary revision
    pub latest_rev: Revision,

    // documents with rev > replica_rev
    pub documents: Vec<Document>,
}

impl ChangesetResponse {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize ChangesetResponse to json")
    }
}

impl std::str::FromStr for ChangesetResponse {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<ChangesetResponse> {
        serde_json::from_str(data).context(anyhow!("Failed to parse ChangesetResponse:\n{}", data))
    }
}

impl fmt::Display for ChangesetResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} ChangesetResponse: rev {}, {} documents since {}]",
            self.arhiv_id,
            self.latest_rev,
            self.documents.len(),
            self.base_rev,
        )
    }
}
