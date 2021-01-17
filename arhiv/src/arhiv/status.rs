use serde::Serialize;

use crate::entities::{Revision, Timestamp};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub arhiv_id: String,
    pub root_dir: String,
    pub is_prime: bool,
    pub rev: Revision,
    pub last_update_time: Timestamp,

    pub committed_documents: u32,
    pub staged_documents: u32,
}
