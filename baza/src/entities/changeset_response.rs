use std::fmt;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::sync::revision::Revision;

use super::Document;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ChangesetResponse {
    // replica revision
    pub base_rev: Revision,

    // primary revision
    pub latest_rev: Revision,

    // documents_history records with rev > replica_rev
    pub new_snapshots: Vec<Document>,

    pub conflicts: Vec<Document>,
}

impl ChangesetResponse {
    #[must_use]
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
            "[ChangesetResponse: rev {}, {} snapshot(s) and {} conflict(s) since {}]",
            self.latest_rev,
            self.new_snapshots.len(),
            self.conflicts.len(),
            self.base_rev,
        )
    }
}
