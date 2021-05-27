use anyhow::*;
use chrono::{DateTime, Local};
use rocket::State;
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{entities::*, Filter, Matcher, OrderBy};

use crate::utils::AppContext;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

#[get("/catalogs/<document_type>")]
pub fn catalog_page(document_type: String, context: State<AppContext>) -> Result<Template> {
    let mut filter = Filter::default();
    filter.matchers.push(Matcher::Type {
        document_type: document_type.clone(),
    });
    filter.page_size = Some(12);
    filter.order.push(OrderBy::UpdatedAt { asc: false });

    let renderer = context.get_renderer();

    let page = context
        .arhiv
        .list_documents(filter)?
        .map(|document| CatalogEntry {
            preview: renderer
                .get_preview(&document)
                .unwrap_or("No preview".to_string()),
            id: document.id,
            document_type: document.document_type,
            updated_at: document.updated_at.into(),
        });

    Ok(Template::render(
        "catalog_page",
        json!({
            "document_type": document_type,
            "page": page,
        }),
    ))
}
