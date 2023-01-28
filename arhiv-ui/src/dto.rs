use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::entities::{DocumentData, Id};
use rs_utils::Timestamp;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum APIRequest {
    #[serde(rename_all = "camelCase")]
    ListDocuments {
        document_types: Vec<String>,
        query: String,
        page: u8,
    },
    GetDocuments {
        ids: Vec<Id>,
    },
    GetStatus {},
    GetDocument {
        id: Id,
    },
    ParseMarkup {
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
pub enum APIResponse {
    ListDocuments {
        documents: Vec<ListDocumentsResult>,
        #[serde(rename = "hasMore")]
        has_more: bool,
    },
    GetDocuments {
        documents: Vec<ListDocumentsResult>,
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
        collections: Vec<DocumentBackref>,
    },
    ParseMarkup {
        ast: Value,
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
