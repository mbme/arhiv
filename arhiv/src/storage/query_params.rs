use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub enum Matcher {
    Field {
        selector: String,
        pattern: String,
    },
    FuzzyField {
        selector: String,
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
