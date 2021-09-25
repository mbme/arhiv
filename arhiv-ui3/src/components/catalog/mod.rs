use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, schema::Collection, Arhiv, Condition, Filter};

pub use self::entries::CatalogConfig;
use self::{entries::CatalogEntries, search::CatalogSearch};
use crate::template_fn;

mod entries;
mod search;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 20;

pub struct Catalog {
    filter: Filter,
    parent_collection: Option<Id>,
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
            parent_collection: None,
            search: None,
            picker_mode: false,
        }
    }

    pub fn from_filter(filter: Filter) -> Self {
        Catalog {
            filter,
            parent_collection: None,
            search: None,
            picker_mode: false,
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

    pub fn show_search(mut self, query_param: Option<&'static str>) -> Self {
        self.search = Some(CatalogSearch { query_param });

        self
    }

    pub fn in_collection(mut self, parent_collection: Option<Id>) -> Self {
        self.parent_collection = parent_collection;

        self
    }

    pub fn picker_mode(mut self) -> Self {
        self.picker_mode = true;

        self
    }

    pub fn render(mut self, arhiv: &Arhiv) -> Result<String> {
        if let Some(ref parent_collection) = self.parent_collection {
            let collection = arhiv.must_get_document(parent_collection)?;
            let data_descripton = arhiv
                .get_schema()
                .get_data_description(collection.document_type)?;

            match data_descripton.collection_of {
                Collection::Type {
                    document_type: _,
                    field,
                } => {
                    self.filter.matchers.push(Condition::Field {
                        field: field.to_string(),
                        pattern: collection.id.to_string(),
                        not: false,
                    });
                }
                _ => {
                    bail!("parent_collection is not a collection");
                }
            };
        }

        let result = arhiv.list_documents(&self.filter)?;

        let pattern = self.filter.get_pattern().unwrap_or_default();
        let document_type = self.filter.get_document_type();

        let search = if let Some(search) = self.search {
            Some(search.render(
                pattern,
                document_type,
                &self.parent_collection,
                self.picker_mode,
            )?)
        } else {
            None
        };

        let mut entries = CatalogEntries::new();
        entries.parent_collection = self.parent_collection.clone();
        if self.picker_mode {
            entries.show_id = true;
            entries.show_type = true;
            entries.title_link = false;
        }

        let entries = entries.render(&result.items, arhiv)?;

        render_template(json!({
            "search": search,
            "parent_collection": self.parent_collection,
            "entries": entries,
            "has_more": result.has_more,
            "next_page_filter": self.filter.get_next_page(),
            "picker_mode": self.picker_mode,
        }))
    }
}
