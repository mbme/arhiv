use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::{
    entities::{DocumentData, DocumentLockKey, Id},
    schema::DataSchema,
};
use rs_utils::{SecretString, Timestamp};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum APIRequest {
    #[serde(rename_all = "camelCase")]
    ListDocuments {
        document_types: Vec<String>,
        query: String,
        page: u8,
        only_conflicts: bool,
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
    CreateAsset {
        file_path: String,
        remove_file: bool,
    },
    Commit {},
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
    CreateArhiv {
        password: SecretString,
    },
    LockArhiv {},
    UnlockArhiv {
        password: Option<SecretString>,
    },
    #[serde(rename_all = "camelCase")]
    ImportKey {
        encrypted_key: String,
        password: SecretString,
    },
    #[serde(rename_all = "camelCase")]
    ExportKey {
        password: SecretString,
        export_password: SecretString,
    },
    CountConflicts {},
}

#[derive(Serialize)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum APIResponse {
    #[serde(rename_all = "camelCase")]
    ListDocuments {
        documents: Vec<ListDocumentsResult>,
        has_more: bool,
        total: usize,
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
        snapshots_count: usize,
        has_conflict: bool,
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
    CreateAsset {
        id: Id,
    },
    #[serde(rename_all = "camelCase")]
    Commit {
        committed_ids: HashSet<Id>,
    },
    #[serde(rename_all = "camelCase")]
    LockDocument {
        lock_key: DocumentLockKey,
    },
    UnlockDocument {},
    ReorderCollectionRefs {},
    CreateArhiv {},
    LockArhiv {},
    UnlockArhiv {},
    ImportKey {},
    #[serde(rename_all = "camelCase")]
    ExportKey {
        key: String,
        qrcode_svg_base64: String,
        html_page: String,
    },
    #[serde(rename_all = "camelCase")]
    CountConflicts {
        conflicts_count: usize,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDocumentsResult {
    pub id: Id,
    pub document_type: String,
    pub title: String,
    pub updated_at: Timestamp,
    pub data: DocumentData,
    pub cover: Option<Id>,
    pub has_conflict: bool,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArhivUIConfig<'a> {
    pub storage_dir: &'a str,
    pub base_path: &'static str,
    pub schema: &'a DataSchema,
    pub use_local_storage: bool,
    pub min_password_length: usize,
    pub arhiv_missing: bool,
    pub arhiv_key_missing: bool,
    pub arhiv_locked: bool,
    pub dev_mode: bool,
    pub arhiv_version: &'a str,
}
