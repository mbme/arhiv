use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv, Filter};

pub use self::entries::CatalogConfig;
use self::{entries::CatalogEntries, search::CatalogSearch};
use crate::template_fn;

mod entries;
mod search;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 20;

pub struct Catalog {
    filter: Filter,
    search: Option<CatalogSearch>,
    picker_mode: bool,
}

impl Catalog {
    pub fn new() -> Self {
        let filter = Filter::default()
            .page_size(PAGE_SIZE)
            .recently_updated_first();

        Catalog {
            filter,
            search: None,
            picker_mode: false,
        }
    }

    pub fn from_filter(filter: Filter) -> Self {
        Catalog {
            filter,
            search: None,
            picker_mode: false,
        }
    }

    pub fn with_type(mut self, document_type: impl AsRef<str>) -> Self {
        self.filter = self.filter.with_document_type(document_type.as_ref());

        self
    }

    pub fn search(mut self, pattern: impl AsRef<str>) -> Self {
        self.filter = self.filter.search(pattern.as_ref());

        self
    }

    pub fn in_collection(mut self, parent_collection: Id) -> Self {
        self.filter = self.filter.with_collection_ref(parent_collection);

        self
    }

    pub fn show_search(mut self, query_param: Option<&'static str>) -> Self {
        self.search = Some(CatalogSearch { query_param });

        self
    }

    pub fn picker_mode(mut self) -> Self {
        self.picker_mode = true;

        self
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let result = arhiv.list_documents(&self.filter)?;

        let pattern = self.filter.get_pattern().unwrap_or_default();
        let document_type = self.filter.get_document_type();
        let parent_collection = self.filter.get_parent_collection();

        let search = if let Some(search) = self.search {
            Some(search.render(pattern, document_type, &parent_collection, self.picker_mode)?)
        } else {
            None
        };

        let mut entries = CatalogEntries::new();
        entries.parent_collection = parent_collection;
        if self.picker_mode {
            entries.show_id = true;
            entries.show_type = true;
            entries.title_link = false;
        }

        let entries = entries.render(&result.items, arhiv)?;

        render_template(json!({
            "search": search,
            "entries": entries,
            "has_more": result.has_more,
            "next_page_filter": self.filter.get_next_page(),
            "picker_mode": self.picker_mode,
        }))
    }
}
