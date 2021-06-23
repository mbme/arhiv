use hyper::{Body, Request};
use serde_json::json;

use crate::{components::Breadcrumbs, utils::render_page};
use rs_utils::server::ServerResponse;

pub async fn new_document_variants_page(_req: Request<Body>) -> ServerResponse {
    let breadcrumbs = Breadcrumbs::NewDocumentVariants.render()?;

    render_page(
        "pages/new_document_variants_page.html.tera",
        json!({
                "breadcrumbs": breadcrumbs, //
        }),
    )
}
