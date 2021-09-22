use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv, Condition, Filter};

pub use self::entries::CatalogConfig;
use self::entries::CatalogEntry;
use crate::template_fn;

mod entries;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 20;

pub struct Catalog {
    filter: Filter,
    parent_collection: Option<Id>,
    show_search: bool,
}

impl Catalog {
    pub fn new() -> Self {
        let filter = Filter::default()
            .page_size(PAGE_SIZE)
            .recently_updated_first();

        Catalog {
            filter,
            parent_collection: None,
            show_search: false,
        }
    }

    pub fn from_filter(filter: Filter) -> Self {
        Catalog {
            filter,
            parent_collection: None,
            show_search: false,
        }
    }

    pub fn with_type(mut self, document_type: impl AsRef<str>) -> Self {
        self.filter = self.filter.with_type(document_type.as_ref());

        self
    }

    pub fn search(mut self, pattern: impl AsRef<str>) -> Self {
        self.filter = self.filter.search(pattern.as_ref());

        self
    }

    pub fn show_search(mut self, show_search: bool) -> Self {
        self.show_search = show_search;

        self
    }

    pub fn with_matcher(mut self, matcher: Condition) -> Self {
        self.filter.matchers.push(matcher);

        self
    }

    pub fn in_collection(mut self, parent_collection: Option<Id>) -> Self {
        self.parent_collection = parent_collection;

        self
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let result = arhiv.list_documents(&self.filter)?;

        let entries = result
            .items
            .into_iter()
            .map(|document| CatalogEntry::new(document, arhiv, &self.parent_collection))
            .collect::<Result<Vec<_>>>()?;

        render_template(json!({
            "show_search": self.show_search,
            "parent_collection": self.parent_collection,
            "pattern": self.filter.get_pattern().unwrap_or_default(),
            "document_type": self.filter.get_document_type().unwrap_or_default(),
            "entries": entries,
            "has_more": result.has_more,
            "next_page_filter": self.filter.get_next_page(),
        }))
    }
}
