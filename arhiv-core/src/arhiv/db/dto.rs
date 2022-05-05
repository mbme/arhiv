use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::entities::{Document, Revision, Timestamp};

pub struct DBSetting<T: Serialize + DeserializeOwned>(pub &'static str, PhantomData<T>);

pub const SETTING_ARHIV_ID: DBSetting<String> = DBSetting("arhiv_id", PhantomData);
pub const SETTING_IS_PRIME: DBSetting<bool> = DBSetting("is_prime", PhantomData);
pub const SETTING_DATA_VERSION: DBSetting<u8> = DBSetting("data_version", PhantomData);
pub const SETTING_LAST_SYNC_TIME: DBSetting<Timestamp> = DBSetting("last_sync_time", PhantomData);

#[derive(Serialize, Deserialize)]
pub struct DbStatus {
    pub arhiv_id: String,
    pub is_prime: bool,
    pub data_version: u8,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
}

#[derive(Serialize, Debug, PartialEq)]
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

#[derive(Serialize, Debug, PartialEq)]
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
