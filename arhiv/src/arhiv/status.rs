use serde::Serialize;

use crate::entities::Revision;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub root_dir: String,
    pub is_prime: bool,
    pub rev: Revision,

    pub committed_documents: u32,
    pub staged_documents: u32,

    pub committed_attachments: u32,
    pub staged_attachments: u32,
}
