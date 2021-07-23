use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::entities::*;

pub struct DBSetting<T: Serialize + DeserializeOwned>(pub &'static str, PhantomData<T>);

pub const SETTING_ARHIV_ID: DBSetting<String> = DBSetting("arhiv_id", PhantomData);
pub const SETTING_IS_PRIME: DBSetting<bool> = DBSetting("is_prime", PhantomData);
pub const SETTING_DB_VERSION: DBSetting<u8> = DBSetting("db_version", PhantomData);
pub const SETTING_LAST_SYNC_TIME: DBSetting<Timestamp> = DBSetting("last_sync_time", PhantomData);

#[derive(Serialize, Deserialize)]
pub struct DbStatus {
    pub arhiv_id: String,
    pub is_prime: bool,
    pub db_version: u8,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DocumentsCount {
    pub documents_committed: u32,
    pub documents_updated: u32,
    pub documents_new: u32,

    pub attachments_committed: u32,
    pub attachments_updated: u32,
    pub attachments_new: u32,

    pub tombstones_committed: u32,
    pub tombstones_updated: u32,
    pub tombstones_new: u32,
}

impl DocumentsCount {
    pub fn count_staged_documents(&self) -> u32 {
        self.documents_updated + self.documents_new
    }

    pub fn count_staged_attachments(&self) -> u32 {
        self.attachments_updated + self.attachments_new
    }

    pub fn count_staged_tombstones(&self) -> u32 {
        self.tombstones_updated + self.tombstones_new
    }

    pub fn count_staged(&self) -> u32 {
        self.count_staged_documents()
            + self.count_staged_attachments()
            + self.count_staged_tombstones()
    }
}

#[derive(Debug, Serialize)]
pub struct ListPage<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

impl<T> ListPage<T> {
    pub fn map<K, F>(self, f: F) -> ListPage<K>
    where
        F: Fn(T) -> K,
    {
        ListPage {
            items: self.items.into_iter().map(f).collect(),
            has_more: self.has_more,
        }
    }
}
