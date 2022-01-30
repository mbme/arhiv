use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::entities::{Id, ERASED_DOCUMENT_TYPE};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Conditions {
    pub field: Option<(String, String)>, // field, pattern
    pub search: Option<String>,          // pattern
    pub document_type: Option<String>,   // document_type
    pub document_ref: Option<Id>,
    pub collection_ref: Option<Id>,
    pub only_staged: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub conditions: Conditions,
    pub order: Vec<OrderBy>,
}

const DEFAULT_PAGE_SIZE: u8 = 20;

impl Default for Filter {
    fn default() -> Self {
        Filter {
            page_offset: Some(0),
            page_size: Some(DEFAULT_PAGE_SIZE),
            conditions: Conditions::default(),
            order: vec![],
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
            conditions: Conditions {
                only_staged: Some(true),
                ..Conditions::default()
            },
            order: vec![],
        }
    }

    #[must_use]
    pub fn all_backrefs(id: impl Into<Id>) -> Filter {
        Filter {
            page_offset: None,
            page_size: None,
            conditions: Conditions {
                document_ref: Some(id.into()),
                ..Conditions::default()
            },
            order: vec![OrderBy::UpdatedAt { asc: false }],
        }
    }

    #[must_use]
    pub fn all_erased_documents() -> Filter {
        Filter {
            page_offset: None,
            page_size: None,
            conditions: Conditions {
                document_type: Some(ERASED_DOCUMENT_TYPE.to_string()),
                ..Conditions::default()
            },
            order: vec![OrderBy::UpdatedAt { asc: false }],
        }
    }

    #[must_use]
    pub fn with_document_type(mut self, document_type: impl Into<String>) -> Filter {
        self.conditions.document_type = Some(document_type.into());

        self
    }

    #[must_use]
    pub fn where_field(mut self, field: impl Into<String>, value: impl Into<String>) -> Filter {
        self.conditions.field = Some((field.into(), value.into()));

        self
    }

    #[must_use]
    pub fn search(mut self, pattern: impl Into<String>) -> Filter {
        let pattern = pattern.into();

        self.conditions.search = if pattern.trim().is_empty() {
            None
        } else {
            Some(pattern)
        };

        self
    }

    #[must_use]
    pub fn on_page(mut self, page: u8) -> Filter {
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

        self.page_size = Some(page_size);
        self.page_offset = Some(page * page_size);

        self
    }

    #[must_use]
    pub fn with_document_ref(mut self, id: Id) -> Filter {
        self.conditions.document_ref = Some(id);

        self
    }

    #[must_use]
    pub fn with_collection_ref(mut self, collection_id: impl Into<Id>) -> Filter {
        self.conditions.collection_ref = Some(collection_id.into());

        self
    }

    #[must_use]
    pub fn only_staged(mut self) -> Filter {
        self.conditions.only_staged = Some(true);

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

    #[must_use]
    pub fn get_current_page(&self) -> u8 {
        let page_offset = self.page_offset.unwrap_or(0);

        self.page_size
            .map_or(0, |page_size| page_offset / page_size)
    }

    #[must_use]
    pub fn get_pattern(&self) -> Option<&str> {
        self.conditions.search.as_deref()
    }

    #[must_use]
    pub fn get_document_type(&self) -> Option<&str> {
        self.conditions.document_type.as_deref()
    }

    #[must_use]
    pub fn get_parent_collection(&self) -> Option<Id> {
        self.conditions.collection_ref.clone()
    }
}
