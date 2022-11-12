use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::entities::Document;

pub struct DBSetting<T: Serialize + DeserializeOwned>(pub &'static str, pub PhantomData<T>);

pub const SETTING_DATA_VERSION: DBSetting<u8> = DBSetting("data_version", PhantomData);

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct DocumentsCount {
    pub documents_committed: u32,
    pub documents_updated: u32,
    pub documents_new: u32,

    pub erased_documents_committed: u32,
    pub erased_documents_staged: u32,

    pub snapshots: u32,
}

impl DocumentsCount {
    #[must_use]
    pub fn count_staged_documents(&self) -> u32 {
        self.documents_updated + self.documents_new
    }

    #[must_use]
    pub fn count_staged(&self) -> u32 {
        self.count_staged_documents() + self.erased_documents_staged
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct BLOBSCount {
    pub blobs_staged: u32,
    pub local_blobs_count: u32,
    pub local_used_blobs_count: u32,
    pub total_blobs_count: u32,
}

#[derive(Debug, Serialize)]
pub struct ListPage {
    pub items: Vec<Document>,
    pub has_more: bool,
}
