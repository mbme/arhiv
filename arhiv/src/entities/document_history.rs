use super::{Document, Revision};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentHistory {
    pub document: Document,
    pub base_rev: Revision,
}

impl DocumentHistory {
    pub fn new(document: Document, base_rev: Revision) -> Self {
        DocumentHistory { document, base_rev }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document history to json")
    }
}
