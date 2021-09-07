use anyhow::*;
use hyper::{Body, Request};
use serde::Serialize;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv, Condition, Filter};
use rs_utils::server::RequestQueryExt;

use self::config::CatalogConfig;
use self::entries::{render_entries, CatalogEntry};
use self::groups::{group_documents, render_groups};
use crate::template_fn;

pub mod config;
mod entries;
mod groups;

template_fn!(render_template, "./catalog.html.tera");

const PAGE_SIZE: u8 = 14;

#[derive(Serialize)]
pub struct Pagination {
    page: u8,
    prev_page_address: Option<String>,
    next_page_address: Option<String>,
}

pub struct Catalog {
    filter: Filter,
    document_type: String,
    pattern: String,
    pagination: Option<Pagination>,
    parent_collection: Option<Id>,
}

impl Catalog {
    pub fn new(document_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        let document_type = document_type.into();
        let pattern = pattern.into();

        let filter = Filter::default()
            .with_type(&document_type)
            .search(&pattern)
            .all_items()
            .recently_updated_first();

        Catalog {
            filter,
            pattern,
            document_type,
            pagination: None,
            parent_collection: None,
        }
    }

    pub fn with_pagination(mut self, req: &Request<Body>) -> Result<Self> {
        let page: u8 = req
            .get_query_param("page")
            .unwrap_or_else(|| "0".to_string())
            .parse()?;

        self.filter.page_size = Some(PAGE_SIZE);
        self.filter.page_offset = Some(PAGE_SIZE * page);

        let prev_page_address = match page {
            0 => None,
            1 => Some(req.get_url_with_updated_query("page", None)),
            _ => Some(req.get_url_with_updated_query("page", Some((page - 1).to_string()))),
        };

        let next_page_address =
            Some(req.get_url_with_updated_query("page", Some((page + 1).to_string())));

        self.pagination = Some(Pagination {
            page,
            prev_page_address,
            next_page_address,
        });

        Ok(self)
    }

    pub fn with_matcher(mut self, matcher: Condition) -> Self {
        self.filter.matchers.push(matcher);

        self
    }

    pub fn in_collection(mut self, collection_id: Id) -> Self {
        self.parent_collection = Some(collection_id);

        self
    }

    pub fn render(mut self, arhiv: &Arhiv, config: &CatalogConfig) -> Result<String> {
        let result = arhiv.list_documents(&self.filter)?;

        if !result.has_more {
            if let Some(pagination) = self.pagination.as_mut() {
                pagination.next_page_address = None;
            }
        }

        let documents = result.items.into_iter().collect::<Vec<_>>();

        let content = if let Some(ref group_by) = config.group_by {
            let data_description = arhiv
                .get_schema()
                .get_data_description(&self.document_type)?;

            let group_names = data_description
                .get_field(group_by.field)?
                .get_enum_values()?;

            let entries = group_documents(
                documents,
                arhiv,
                config,
                group_by.field,
                &self.parent_collection,
            )?;

            render_groups(group_names, entries, group_by)?
        } else {
            let items = documents
                .into_iter()
                .map(|document| CatalogEntry::new(document, arhiv, config, &self.parent_collection))
                .collect::<Result<Vec<_>>>()?;

            render_entries(&items)?
        };

        render_template(json!({
            "content": content,
            "pattern": self.pattern,
            "pagination": self.pagination,
            "document_type": self.document_type,
        }))
    }
}
