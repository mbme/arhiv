use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::entities::Id;

#[derive(Debug, Serialize, Deserialize)]
pub enum Condition {
    Field {
        field: String,
        pattern: String,
        not: bool,
    },
    Search {
        pattern: String,
    },
    Type {
        document_type: String,
    },
    Ref {
        id: Id,
    },
    NotCollectionChild {
        child_document_type: String,
        child_collection_field: String,
        collection_id: Id,
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
    Relevant,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub matchers: Vec<Condition>,
    pub order: Vec<OrderBy>,
    pub mode: FilterMode,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            page_offset: Some(0),
            page_size: Some(20),
            matchers: vec![],
            order: vec![],
            mode: FilterMode::Relevant,
        }
    }
}

impl AsRef<Filter> for Filter {
    fn as_ref(&self) -> &Filter {
        self
    }
}

impl Filter {
    #[must_use]
    pub fn all_staged_documents() -> Filter {
        Filter {
            page_offset: None,
            page_size: None,
            matchers: vec![],
            mode: FilterMode::Staged,
            order: vec![],
        }
    }

    pub fn backrefs(id: impl Into<Id>) -> Filter {
        let id = id.into();

        Filter {
            page_offset: None,
            page_size: None,
            mode: FilterMode::Relevant,
            matchers: vec![Condition::Ref { id }],
            order: vec![OrderBy::UpdatedAt { asc: false }],
        }
    }

    pub fn with_type(mut self, document_type: impl Into<String>) -> Filter {
        self.matchers.push(Condition::Type {
            document_type: document_type.into(),
        });

        self
    }

    pub fn search(mut self, pattern: impl Into<String>) -> Filter {
        self.matchers.push(Condition::Search {
            pattern: pattern.into(),
        });

        self
    }

    #[must_use]
    pub fn with_matcher(mut self, matcher: Condition) -> Filter {
        self.matchers.push(matcher);

        self
    }

    #[must_use]
    pub fn all_items(mut self) -> Filter {
        self.page_size = None;
        self.page_offset = None;

        self
    }

    #[must_use]
    pub fn recently_updated_first(mut self) -> Filter {
        self.order.push(OrderBy::UpdatedAt { asc: false });

        self
    }

    #[must_use]
    pub fn page_size(mut self, page: u8) -> Filter {
        self.page_size = Some(page);

        self
    }
}
