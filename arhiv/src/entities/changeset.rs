use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Document, Id, Revision};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    pub arhiv_id: String,
    pub base_rev: Revision,
    pub documents: Vec<Document>,
}

impl Changeset {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize Changeset to json")
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn contains(&self, id: &Id) -> bool {
        self.documents
            .iter()
            .find(|document| &document.id == id)
            .is_some()
    }
}

impl fmt::Display for Changeset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} Changeset: {} documents since {}]",
            self.arhiv_id,
            self.documents.len(),
            self.base_rev,
        )
    }
}
