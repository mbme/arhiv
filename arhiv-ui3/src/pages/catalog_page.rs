use chrono::{DateTime, Local};
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use crate::{components::Catalog, ui_config::CatalogConfig, utils::render_page};
use arhiv_core::{entities::*, Arhiv};
use rs_utils::server::{RequestQueryExt, ServerResponse};

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

pub async fn catalog_page(req: Request<Body>) -> ServerResponse {
    let document_type: &String = req.param("document_type").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let pattern = req.get_query_param("pattern").unwrap_or("".to_string());

    let catalog = Catalog::new(document_type, pattern)
        .with_pagination(&req)?
        .render(arhiv, CatalogConfig::get_config(document_type))?;

    render_page(
        "pages/catalog_page.html.tera",
        json!({
            "document_type": document_type,
            "catalog": catalog,
        }),
    )
}
