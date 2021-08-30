use std::collections::HashMap;

use anyhow::*;
use arhiv_core::entities::Document;
use hyper::{Body, Request};
use serde::Serialize;
use serde_json::json;

pub mod config;
mod entry;

use crate::template_fn;
use arhiv_core::{Arhiv, Condition, Filter};
use rs_utils::server::RequestQueryExt;

use self::config::{CatalogConfig, CatalogGroupBy};
use self::entry::CatalogEntry;

template_fn!(render_template, "./catalog.html.tera");
template_fn!(render_entries_template, "./catalog_entries.html.tera");
template_fn!(render_groups_template, "./catalog_groups.html.tera");

#[derive(Serialize)]
struct CatalogGroup {
    value: &'static str,
    open: bool,
    items: String,
    items_count: usize,
}

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
    new_document_query: String,
    document_url_query: String,
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
            new_document_query: "".to_string(),
            document_url_query: "".to_string(),
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

    pub fn with_new_document_query(mut self, mut query: String) -> Self {
        if !query.is_empty() {
            query.insert(0, '?');
        }

        self.new_document_query = query;

        self
    }

    pub fn with_document_url_query(mut self, mut query: String) -> Self {
        if !query.is_empty() {
            query.insert(0, '?');
        }

        self.document_url_query = query;

        self
    }

    pub fn render(mut self, arhiv: &Arhiv, config: &CatalogConfig) -> Result<String> {
        let result = arhiv.list_documents(self.filter)?;

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

            let documents = group_documents(documents, group_by.field)?;
            let entries = documents_to_entries(documents, arhiv, config)?;

            let group_names = data_description
                .get_field(group_by.field)?
                .get_enum_values()?;

            render_groups(group_names, entries, &self.document_url_query, group_by)?
        } else {
            let items = documents
                .into_iter()
                .map(|document| CatalogEntry::new(document, arhiv, config))
                .collect::<Result<Vec<_>>>()?;

            render_entries(&items, &self.document_url_query)?
        };

        render_template(json!({
            "content": content,
            "pattern": self.pattern,
            "pagination": self.pagination,
            "document_type": self.document_type,
        }))
    }
}

fn render_entries(entries: &[CatalogEntry], query: &str) -> Result<String> {
    render_entries_template(json!({
        "items": entries,
        "query": query,
    }))
}

fn group_documents(
    documents: Vec<Document>,
    field: &str,
) -> Result<HashMap<String, Vec<Document>>> {
    let mut result = HashMap::new();

    for document in documents {
        let key = document
            .data
            .get_str(field)
            .ok_or_else(|| anyhow!("can't find field"))?
            .to_string();

        let entry = result.entry(key).or_insert_with(Vec::new);

        entry.push(document);
    }

    Ok(result)
}

fn documents_to_entries(
    documents: HashMap<String, Vec<Document>>,
    arhiv: &Arhiv,
    config: &CatalogConfig,
) -> Result<HashMap<String, Vec<CatalogEntry>>> {
    documents
        .into_iter()
        .map(|(group_name, documents)| {
            let entries = documents
                .into_iter()
                .map(|document| CatalogEntry::new(document, arhiv, config))
                .collect::<Result<Vec<_>>>()?;

            Ok((group_name, entries))
        })
        .collect::<Result<HashMap<_, _>>>()
}

fn render_groups(
    group_names: &[&'static str],
    mut documents: HashMap<String, Vec<CatalogEntry>>,
    query: &str,
    group_by: &CatalogGroupBy,
) -> Result<String> {
    let mut groups = group_names
        .iter()
        .map(|group_name| {
            let entries = documents.remove(*group_name).unwrap_or_default();
            let items_count = entries.len();
            let items = render_entries(&entries, query)?;

            Ok(CatalogGroup {
                value: group_name,
                items,
                items_count,
                open: group_by.open_groups.contains(group_name),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // skip empty groups if needed
    if group_by.skip_empty_groups {
        groups.retain(|group| !group.items.is_empty());
    }

    // open first non-empty group if no groups open yet
    if !groups.iter().any(|group| group.open) {
        if let Some(group) = groups.iter_mut().find(|group| !group.items.is_empty()) {
            group.open = true;
        }
    }

    render_groups_template(json!({ "groups": groups }))
}
