use anyhow::*;
use chrono::{DateTime, Local};
use rocket::{uri, State};
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{entities::*, Filter, Matcher, OrderBy};

use crate::{components::prepare_catalog_values, utils::AppContext};

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

    let result = context.arhiv.list_documents(filter)?;
    let components_catalog = prepare_catalog_values(&context.get_renderer(), result.items)?;

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
        "pages/catalog_page",
        json!({
            "document_type": document_type,
            "components_catalog": components_catalog,
            "prev_link": prev_link,
            "page": page,
            "next_link": next_link,
        }),
    ))
}
