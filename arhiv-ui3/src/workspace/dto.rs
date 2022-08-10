use serde::{Deserialize, Serialize};

use arhiv_core::entities::Id;
use rs_utils::Timestamp;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceRequest {
    ListDocuments { query: String },
    GetStatus {},
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResult {
    pub id: Id,
    pub document_type: String,
    pub title: String,
    pub updated_at: Timestamp,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceResponse {
    ListDocuments {
        documents: Vec<ListDocumentsResult>,
        #[serde(rename = "hasMore")]
        has_more: bool,
    },
    GetStatus {
        status: String,
    },
}
