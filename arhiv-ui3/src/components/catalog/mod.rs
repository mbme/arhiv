use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv, Condition, Filter};

pub use self::entries::CatalogConfig;
use self::entries::CatalogEntry;
use crate::{template_fn, urls::catalog_fragment_url};

mod entries;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 20;

pub struct Catalog {
    filter: Filter,
    parent_collection: Option<Id>,
}

impl Catalog {
    pub fn new(document_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        let document_type = document_type.into();
        let pattern = pattern.into();

        let filter = Filter::default()
            .with_type(&document_type)
            .page_size(PAGE_SIZE)
            .search(&pattern)
            .recently_updated_first();

        Catalog {
            filter,
            parent_collection: None,
        }
    }

    pub fn from_filter(filter: Filter) -> Self {
        Catalog {
            filter,
            parent_collection: None,
        }
    }

    pub fn on_page(mut self, page: u8) -> Self {
        self.filter.page_offset = Some(PAGE_SIZE * page);

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
            "entries": entries,
            "has_more": result.has_more,
            "url": catalog_fragment_url(&self.parent_collection),
            "next_page_filter": self.filter.get_next_page(),
        }))
    }
}
