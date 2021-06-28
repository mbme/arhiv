use arhiv_core::schema::SCHEMA;
use hyper::{Body, Request};
use serde_json::json;

use crate::{components::Breadcrumbs, utils::render_page};
use rs_utils::server::ServerResponse;

pub async fn new_document_variants_page(_req: Request<Body>) -> ServerResponse {
    let breadcrumbs = Breadcrumbs::NewDocumentVariants.render()?;

    let document_types = SCHEMA
        .modules
        .iter()
        .filter(|module| !module.is_internal)
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    render_page(
        "pages/new_document_variants_page.html.tera",
        json!({
            "breadcrumbs": breadcrumbs, //
            "document_types": document_types,
        }),
    )
}
