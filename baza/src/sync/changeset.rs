use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, InstanceId, Revision};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChangesetRequest {
    pub instance_id: InstanceId,
    pub data_version: u8,
    pub rev: Revision,
}

impl fmt::Display for ChangesetRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[ChangesetRequest from {}: {}]",
            self.instance_id,
            self.rev.serialize(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Changeset {
    pub data_version: u8,
    pub documents: Vec<Document>,
}

impl Changeset {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    #[must_use]
    pub fn contains(&self, id: &Id) -> bool {
        self.documents.iter().any(|document| &document.id == id)
    }
}

impl fmt::Display for Changeset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Changeset (schema version {}) : {} documents]",
            self.data_version,
            self.documents.len(),
        )
    }
}
