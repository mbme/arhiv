use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::entities::Id;

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
    Ref {
        id: Id,
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

impl Filter {
    pub fn all_staged_documents() -> Filter {
        Filter {
            page_offset: None,
            page_size: None,
            matchers: vec![],
            mode: Some(FilterMode::Staged),
            order: vec![],
        }
    }

    pub fn backrefs(id: impl Into<Id>) -> Filter {
        let id = id.into();

        Filter {
            page_offset: None,
            page_size: None,
            mode: None,
            matchers: vec![Matcher::Ref { id }],
            order: vec![OrderBy::UpdatedAt { asc: false }],
        }
    }

    pub fn with_type(mut self, document_type: impl Into<String>) -> Filter {
        self.matchers.push(Matcher::Type {
            document_type: document_type.into(),
        });

        self
    }

    pub fn search(mut self, pattern: impl Into<String>) -> Filter {
        self.matchers.push(Matcher::Search {
            pattern: pattern.into(),
        });

        self
    }

    pub fn all_items(mut self) -> Filter {
        self.page_size = None;
        self.page_offset = None;

        self
    }

    pub fn recently_updated_first(mut self) -> Filter {
        self.order.push(OrderBy::UpdatedAt { asc: false });

        self
    }

    pub fn page_size(mut self, page: u8) -> Filter {
        self.page_size = Some(page);

        self
    }
}
