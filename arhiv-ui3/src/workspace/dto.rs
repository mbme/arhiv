use serde::{Deserialize, Serialize};

use arhiv_core::entities::{DocumentData, Id};
use rs_utils::Timestamp;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceRequest {
    ListDocuments { query: String },
    GetStatus {},
    GetDocument { id: Id },
    RenderMarkup { markup: String },
    GetRef { id: Id },
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
    #[serde(rename_all = "camelCase")]
    GetDocument {
        id: Id,
        document_type: String,
        subtype: String,
        updated_at: Timestamp,
        data: DocumentData,
    },
    RenderMarkup {
        html: String,
    },
    #[serde(rename_all = "camelCase")]
    GetRef {
        id: Id,
        document_type: String,
        subtype: String,
        title: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResult {
    pub id: Id,
    pub document_type: String,
    pub subtype: String,
    pub title: String,
    pub updated_at: Timestamp,
}
