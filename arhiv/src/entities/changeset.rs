use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Attachment, Document, Revision};

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
