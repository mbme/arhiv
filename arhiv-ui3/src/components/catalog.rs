use anyhow::*;
use hyper::{Body, Request};
use serde::Serialize;
use serde_json::json;

use crate::{markup::MarkupStringExt, templates::TEMPLATES};
use arhiv_core::{entities::*, markup::MarkupStr, schema::SCHEMA, Arhiv, Filter, Matcher, OrderBy};
use rs_utils::server::RequestQueryExt;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    title: String,
    preview: Option<String>,
    fields: Vec<(&'static str, String)>,
}

impl CatalogEntry {
    pub fn new(document: Document, arhiv: &Arhiv, config: &CatalogConfig) -> Result<Self> {
        let data_description = SCHEMA.get_data_description(&document.document_type)?;

        let title_field = data_description.pick_title_field()?;

        let title = document
            .get_field_str(title_field.name)
            .ok_or(anyhow!("title field missing"))?;

        let mut preview = None;

        if let Some(preview_field) = config.preview {
            let markup: MarkupStr = document
                .get_field_str(preview_field)
                .ok_or(anyhow!("preview field missing"))?
                .into();

            preview = Some(markup.preview(4).to_html(arhiv))
        }

        let fields = config
            .fields
            .iter()
            .map(|field| {
                (
                    *field,
                    document.get_field_str(field).unwrap_or("").to_string(),
                )
            })
            .collect();

        Ok(CatalogEntry {
            title: title.to_string(),
            id: document.id,
            document_type: document.document_type,
            preview,
            fields,
        })
    }
}

#[derive(Serialize)]
struct CatalogGroup {
    field: &'static str,
    value: &'static str,
    open: bool,
    items: Vec<CatalogEntry>,
}

pub struct CatalogConfig {
    pub group_by: Option<CatalogGroupBy>,
    pub preview: Option<&'static str>,
    pub fields: Vec<&'static str>,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        CatalogConfig {
            group_by: None,
            preview: None,
            fields: vec![],
        }
    }
}

pub struct CatalogGroupBy {
    pub field: &'static str,
    pub open_groups: Vec<&'static str>,
    pub skip_empty_groups: bool,
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
            new_document_query: "".to_string(),
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

    pub fn with_new_document_query(mut self, mut query: String) -> Self {
        if !query.is_empty() {
            query.insert_str(0, "?");
        }

        self.new_document_query = query;

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

        let data_description = SCHEMA.get_data_description(&self.document_type)?;

        if let Some(ref group_by) = config.group_by {
            groups = data_description
                .get_field(group_by.field)?
                .get_enum_values()?
                .into_iter()
                .map(|enum_value| CatalogGroup {
                    field: group_by.field,
                    value: enum_value,
                    open: false,
                    items: vec![],
                })
                .collect();

            for document in result.items.into_iter() {
                let key = document
                    .get_field_str(group_by.field)
                    .ok_or(anyhow!("can't find field"))?;

                let mut group = groups
                    .iter_mut()
                    .find(|group| group.value == key)
                    .ok_or(anyhow!("can't find group"))?;

                group.open = group_by.open_groups.contains(&group.value);

                group
                    .items
                    .push(CatalogEntry::new(document, arhiv, &config)?);
            }

            // skip empty groups if needed
            if group_by.skip_empty_groups {
                groups.retain(|group| !group.items.is_empty());
            }

            // open first non-empty group if no groups open yet
            if groups.iter().find(|group| group.open).is_none() {
                groups
                    .iter_mut()
                    .find(|group| !group.items.is_empty())
                    .map(|group| group.open = true);
            }
        } else {
            items = result
                .items
                .into_iter()
                .map(|document| CatalogEntry::new(document, arhiv, &config))
                .collect::<Result<_>>()?;
        }

        TEMPLATES.render(
            "components/catalog.html.tera",
            json!({
                "items": items,
                "groups": groups,
                "pattern": self.pattern,
                "pagination": self.pagination,
                "document_type": self.document_type,
                "is_internal_type": data_description.is_internal,
                "new_document_query": self.new_document_query,
            }),
        )
    }
}
