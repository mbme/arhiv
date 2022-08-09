use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceRequest {
    ListDocuments { query: Option<String> },
    GetStatus {},
}

#[derive(Serialize)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceResponse {
    ListDocuments { documents: Vec<String> },
    GetStatus { status: String },
}
