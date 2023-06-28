use std::fmt;

use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id};

use super::revision::Revision;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Changeset {
    pub data_version: u8,
    pub base_rev: Revision,
    pub documents: Vec<Document>,
}

impl Changeset {
    #[must_use]
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize Changeset to json")
    }

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
            "[Changeset (schema version {}) : {} documents since {}]",
            self.data_version,
            self.documents.len(),
            self.base_rev,
        )
    }
}
