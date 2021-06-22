use anyhow::*;
use chrono::{DateTime, Local};
use hyper::{Body, Request};
use serde::Serialize;
use serde_json::json;

use crate::{markup::ArhivMarkupExt, templates::TEMPLATES, ui_config::CatalogConfig};
use arhiv_core::{entities::*, schema::SCHEMA, Arhiv, Filter, Matcher, OrderBy};
use rs_utils::server::RequestQueryExt;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

#[derive(Serialize)]
struct CatalogGroup {
    name: String,
    open: bool,
    items: Vec<CatalogEntry>,
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
}

impl Catalog {
    pub fn new(document_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        let document_type = document_type.into();
        let pattern = pattern.into();

        let mut filter = Filter::default();

        filter.matchers.push(Matcher::Type {
            document_type: document_type.clone(),
        });
        filter.matchers.push(Matcher::Search {
            pattern: pattern.clone(),
        });
        filter.page_size = None;
        filter.page_offset = None;
        filter.order.push(OrderBy::UpdatedAt { asc: false });

        Catalog {
            filter,
            pattern,
            document_type,
            pagination: None,
        }
    }

    pub fn with_pagination(mut self, req: &Request<Body>) -> Result<Self> {
        let page: u8 = req
            .get_query_param("page")
            .unwrap_or("0".to_string())
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

    pub fn with_matcher(mut self, matcher: Matcher) -> Self {
        self.filter.matchers.push(matcher);

        self
    }

    pub fn render(mut self, arhiv: &Arhiv, config: CatalogConfig) -> Result<String> {
        let result = arhiv.list_documents(self.filter)?;

        if !result.has_more {
            if let Some(pagination) = self.pagination.as_mut() {
                pagination.next_page_address = None;
            }
        }

        let mut items: Vec<CatalogEntry> = vec![];
        let mut groups: Vec<CatalogGroup> = vec![];

        if let Some(group_by) = config.group_by {
            groups = SCHEMA
                .get_data_description_by_type(&self.document_type)?
                .get_field(group_by.field)?
                .get_enum_values()?
                .into_iter()
                .map(|enum_value| CatalogGroup {
                    name: enum_value.to_string(),
                    open: group_by.open_groups.contains(enum_value),
                    items: vec![],
                })
                .collect();

            for document in result.items.into_iter() {
                let key = document
                    .get_field_str(group_by.field)
                    .ok_or(anyhow!("can't find field"))?;

                let group = groups
                    .iter_mut()
                    .find(|group| group.name == key)
                    .ok_or(anyhow!("can't find group"))?;

                group.items.push(CatalogEntry {
                    preview: arhiv.render_preview(&document),
                    id: document.id,
                    document_type: document.document_type,
                    updated_at: document.updated_at.into(),
                });
            }
        } else {
            items = result
                .items
                .into_iter()
                .map(|document| CatalogEntry {
                    preview: arhiv.render_preview(&document),
                    id: document.id,
                    document_type: document.document_type,
                    updated_at: document.updated_at.into(),
                })
                .collect();
        }

        TEMPLATES.render(
            "components/catalog.html.tera",
            json!({
                "items": items,
                "groups": groups,
                "pattern": self.pattern,
                "pagination": self.pagination,
            }),
        )
    }
}
