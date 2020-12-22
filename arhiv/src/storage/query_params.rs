use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct Matcher {
    pub selector: String,
    pub pattern: String,
    pub fuzzy: bool,
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
pub enum DocumentFilterMode {
    Archived,
    Staged,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFilter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub matchers: Vec<Matcher>,
    pub order: Vec<OrderBy>,
    pub mode: Option<DocumentFilterMode>,
}

impl Default for DocumentFilter {
    fn default() -> Self {
        DocumentFilter {
            page_offset: Some(0),
            page_size: Some(20),
            matchers: vec![],
            order: vec![],
            mode: None,
        }
    }
}

pub const DOCUMENT_FILTER_STAGED: DocumentFilter = DocumentFilter {
    page_offset: None,
    page_size: None,
    matchers: vec![],
    mode: Some(DocumentFilterMode::Staged),
    order: vec![],
};

pub struct AttachmentFilter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
}

impl Default for AttachmentFilter {
    fn default() -> Self {
        AttachmentFilter {
            page_offset: Some(0),
            page_size: Some(20),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPage<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}
