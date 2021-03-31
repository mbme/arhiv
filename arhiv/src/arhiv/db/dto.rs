use crate::entities::*;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Deserialize)]
pub struct DbStatus {
    pub arhiv_id: String,
    pub is_prime: bool,
    pub schema_version: u8,
    pub db_version: u8,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
}

impl DbStatus {
    pub fn get_prime_status(&self) -> &str {
        if self.is_prime {
            "prime"
        } else {
            "replica"
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DocumentsCount {
    pub documents_committed: u32,
    pub documents_updated: u32,
    pub documents_new: u32,
    pub attachments_committed: u32,
    pub attachments_updated: u32,
    pub attachments_new: u32,
}

impl DocumentsCount {
    pub fn count_staged_documents(&self) -> u32 {
        self.documents_updated + self.documents_new
    }

    pub fn count_staged_attachments(&self) -> u32 {
        self.attachments_updated + self.attachments_new
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Matcher {
    Field {
        selector: String,
        pattern: String,
    },
    Search {
        pattern: String,
    },
    Type {
        #[serde(rename = "documentType")]
        document_type: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderBy {
    Field {
        selector: String,
        asc: bool,
    },
    EnumField {
        selector: String,
        asc: bool,
        #[serde(rename = "enumOrder")]
        enum_order: Vec<String>,
    },
    UpdatedAt {
        asc: bool,
    },
}

impl Default for OrderBy {
    fn default() -> Self {
        OrderBy::UpdatedAt { asc: true }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterMode {
    Archived,
    Staged,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub matchers: Vec<Matcher>,
    pub order: Vec<OrderBy>,
    pub mode: Option<FilterMode>,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            page_offset: Some(0),
            page_size: Some(20),
            matchers: vec![],
            order: vec![],
            mode: None,
        }
    }
}

pub const DOCUMENT_FILTER_STAGED: Filter = Filter {
    page_offset: None,
    page_size: None,
    matchers: vec![],
    mode: Some(FilterMode::Staged),
    order: vec![],
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
