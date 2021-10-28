use anyhow::*;
use maud::Render;
use rs_utils::server::Url;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv, Filter};

pub use self::entries::{CatalogConfig, CatalogEntries};
use self::pagination::render_pagination;
pub use self::search_input::render_search_input;
use crate::template_fn;

mod entries;
mod pagination;
mod search_input;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 20;

pub struct Catalog {
    url: Url,
    filter: Filter,
    picker_mode: bool,
}

impl Catalog {
    pub fn new(url: Url) -> Self {
        let mut filter = Filter::default()
            .page_size(PAGE_SIZE)
            .recently_updated_first();

        if let Some(pattern) = url.get_query_param("pattern") {
            filter = filter.search(pattern);
        }

        if let Some(page) = url.get_query_param("page") {
            let page: u8 = page.parse().expect("page must be u8");
            filter = filter.on_page(page);
        }

        Catalog {
            url,
            filter,
            picker_mode: false,
        }
    }

    pub fn with_type(mut self, document_type: impl AsRef<str>) -> Self {
        self.filter = self.filter.with_document_type(document_type.as_ref());

        self
    }

    pub fn in_collection(mut self, parent_collection: impl Into<Id>) -> Self {
        self.filter = self.filter.with_collection_ref(parent_collection);

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
        let current_page = self.filter.get_current_page();

        let pagination = render_pagination(&self.url, current_page, result.has_more)
            .render()
            .into_string();

        let search_input = render_search_input(
            pattern,
            document_type,
            &self.url.render(),
            !self.picker_mode,
        )
        .render()
        .into_string();

        let mut entries = CatalogEntries::new();
        entries.parent_collection = parent_collection;
        if self.picker_mode {
            entries.show_id = true;
            entries.show_type = true;
            entries.title_link = false;
        }

        let items = result.items.iter().collect::<Vec<_>>();
        let entries = entries.render(&items, arhiv)?;

        render_template(json!({
            "search_input": search_input,
            "entries": entries,
            "pagination": pagination,
        }))
    }
}
