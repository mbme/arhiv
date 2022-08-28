use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use arhiv_core::entities::{DocumentData, Id};
use rs_utils::Timestamp;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum WorkspaceRequest {
    ListDocuments {
        query: String,
        page: u8,
    },
    GetStatus {},
    GetDocument {
        id: Id,
    },
    RenderMarkup {
        markup: String,
    },
    #[serde(rename_all = "camelCase")]
    CreateDocument {
        document_type: String,
        subtype: String,
        data: DocumentData,
    },
    SaveDocument {
        id: Id,
        subtype: String,
        data: DocumentData,
    },
    EraseDocument {
        id: Id,
    },
    #[serde(rename_all = "camelCase")]
    ListDir {
        dir: Option<String>,
        show_hidden: bool,
    },
    #[serde(rename_all = "camelCase")]
    CreateAttachment {
        file_path: String,
    },
    Scrape {
        url: String,
    },
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
        title: String,
        document_type: String,
        subtype: String,
        updated_at: Timestamp,
        data: DocumentData,
        backrefs: Vec<DocumentBackref>,
    },
    RenderMarkup {
        html: String,
    },
    CreateDocument {
        id: Option<Id>,
        errors: Option<SaveDocumentErrors>,
    },
    SaveDocument {
        errors: Option<SaveDocumentErrors>,
    },
    EraseDocument {},
    ListDir {
        dir: String,
        entries: Vec<DirEntry>,
    },
    CreateAttachment {
        id: Id,
    },
    Scrape {
        documents: Vec<ListDocumentsResult>,
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

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentErrors {
    pub document_errors: Vec<String>,
    pub field_errors: HashMap<String, Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentBackref {
    pub id: Id,
    pub document_type: String,
    pub subtype: String,
    pub title: String,
}

#[derive(Serialize)]
#[serde(tag = "typeName")]
pub enum DirEntry {
    #[serde(rename_all = "camelCase")]
    Dir {
        name: String,
        path: String,
        is_readable: bool,
    },

    #[serde(rename_all = "camelCase")]
    File {
        name: String,
        path: String,
        is_readable: bool,
        size: u64,
    },

    #[serde(rename_all = "camelCase")]
    Symlink {
        name: String,
        path: String,
        is_readable: bool,
        links_to: String,
        size: Option<u64>,
    },
}

impl DirEntry {
    pub fn get_name(&self) -> &str {
        match self {
            DirEntry::Dir { name, .. } => name,
            DirEntry::File { name, .. } => name,
            DirEntry::Symlink { name, .. } => name,
        }
    }
}
