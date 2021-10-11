use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::entities::Id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conditions {
    pub field: Option<(String, String)>, // field, pattern
    pub search: Option<String>,          // pattern
    pub document_type: Option<String>,   // document_type
    pub document_ref: Option<Id>,
    pub collection_ref: Option<Id>,
    pub only_staged: Option<bool>,
}

impl Default for Conditions {
    fn default() -> Self {
        Conditions {
            field: None,
            search: None,
            document_type: None,
            document_ref: None,
            collection_ref: None,
            only_staged: None,
        }
    }
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

impl Default for Filter {
    fn default() -> Self {
        Filter {
            page_offset: Some(0),
            page_size: Some(20),
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
    pub fn backrefs(id: impl Into<Id>) -> Filter {
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
        self.conditions.search = Some(pattern.into());

        self
    }

    #[must_use]
    pub fn with_document_ref(mut self, id: Id) -> Filter {
        self.conditions.document_ref = Some(id);

        self
    }

    #[must_use]
    pub fn with_collection_ref(mut self, collection_id: Id) -> Filter {
        self.conditions.collection_ref = Some(collection_id);

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
    pub fn get_next_page(&self) -> Option<Filter> {
        match (self.page_size, self.page_offset) {
            (Some(page_size), Some(page_offset)) => {
                let mut next_page = self.clone();

                next_page.page_offset = Some(page_offset + page_size);

                Some(next_page)
            }
            _ => None,
        }
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
