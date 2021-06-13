use hyper::{Body, Request};
use serde_json::json;

use crate::utils::render_page;
use rs_utils::server::ServerResponse;

pub async fn new_document_variants_page(_req: Request<Body>) -> ServerResponse {
    render_page("pages/new_document_variants_page.html.tera", json!({}))
}
