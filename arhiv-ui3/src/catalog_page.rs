use anyhow::*;
use chrono::{DateTime, Local};
use rocket::State;
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{entities::*, markup::MarkupRenderer, Arhiv, Filter, Matcher, OrderBy};

use crate::utils::TemplateContext;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

#[get("/catalogs/<document_type>")]
pub fn catalog_page(
    document_type: String,
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<Template> {
    let mut filter = Filter::default();
    filter.matchers.push(Matcher::Type {
        document_type: document_type.clone(),
    });
    filter.page_size = Some(12);
    filter.order.push(OrderBy::UpdatedAt { asc: false });

    let renderer = MarkupRenderer::new(&arhiv, &context.markup_render_options);

    let page = arhiv.list_documents(filter)?.map(|document| CatalogEntry {
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
