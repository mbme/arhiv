use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::entities::{BLOBId, DocumentData, DocumentLockKey, Id};
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
        data: DocumentData,
        collections: Vec<Id>,
    },
    #[serde(rename_all = "camelCase")]
    SaveDocument {
        lock_key: DocumentLockKey,
        id: Id,
        data: DocumentData,
        collections: Vec<Id>,
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
        move_file: bool,
    },
    Commit {},
    Sync {},
    GetSaveState {},
    LockDocument {
        id: Id,
    },
    #[serde(rename_all = "camelCase")]
    UnlockDocument {
        id: Id,
        lock_key: Option<DocumentLockKey>,
        force_unlock: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    ReorderCollectionRefs {
        collection_id: Id,
        id: Id,
        new_pos: usize,
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
        documents: Vec<GetDocumentsResult>,
    },
    GetStatus {
        status: String,
    },
    #[serde(rename_all = "camelCase")]
    GetDocument {
        id: Id,
        title: String,
        document_type: String,
        updated_at: Timestamp,
        data: DocumentData,
        refs: Vec<Id>,
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
    Commit {},
    Sync {},
    #[serde(rename_all = "camelCase")]
    GetSaveState {
        can_commit: bool,
        can_sync: bool,
    },
    #[serde(rename_all = "camelCase")]
    LockDocument {
        lock_key: DocumentLockKey,
    },
    UnlockDocument {},
    ReorderCollectionRefs {},
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResult {
    pub id: Id,
    pub document_type: String,
    pub title: String,
    pub updated_at: Timestamp,
    pub data: DocumentData,
    pub cover: Option<BLOBId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentsResult {
    pub id: Id,
    pub document_type: String,
    pub title: String,
    pub updated_at: Timestamp,
    pub data: DocumentData,
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
