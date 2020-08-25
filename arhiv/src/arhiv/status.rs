use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub is_prime: bool,
    pub rev: u32,

    pub commited_documents: u32,
    pub staged_documents: u32,

    pub commited_attachments: u32,
    pub staged_attachments: u32,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).expect("Failed to serialize status to json")
        )
    }
}
