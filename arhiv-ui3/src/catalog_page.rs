use anyhow::*;
use chrono::{DateTime, Local};
use rocket::{uri, State};
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{entities::*, Filter, Matcher, OrderBy};

use crate::utils::AppContext;

const PAGE_SIZE: u8 = 14;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

#[get("/catalogs/<document_type>?<page>")]
pub fn catalog_page(
    document_type: String,
    page: Option<u8>,
    context: State<AppContext>,
) -> Result<Template> {
    let page = page.unwrap_or(0);

    let mut filter = Filter::default();
    filter.matchers.push(Matcher::Type {
        document_type: document_type.clone(),
    });
    filter.page_size = Some(PAGE_SIZE);
    filter.page_offset = Some(PAGE_SIZE * page);
    filter.order.push(OrderBy::UpdatedAt { asc: false });

    let renderer = context.get_renderer();

    let result = context.arhiv.list_documents(filter)?;

    let items: Vec<_> = result
        .items
        .into_iter()
        .map(|document| CatalogEntry {
            preview: renderer
                .get_preview(&document)
                .unwrap_or("No preview".to_string()),
            id: document.id,
            document_type: document.document_type,
            updated_at: document.updated_at.into(),
        })
        .collect();

    let prev_link = match page {
        0 => None,
        1 => Some(uri!(catalog_page: &document_type, _).to_string()),
        _ => Some(uri!(catalog_page: &document_type, page - 1).to_string()),
    };

    let next_link = if result.has_more {
        Some(uri!(catalog_page: &document_type, page + 1).to_string())
    } else {
        None
    };

    Ok(Template::render(
        "catalog_page",
        json!({
            "document_type": document_type,
            "items": items,
            "prev_link": prev_link,
            "page": page,
            "next_link": next_link,
        }),
    ))
}
