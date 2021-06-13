use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{app_context::AppContext, utils::render_page};
use rs_utils::server::ServerResponse;

pub async fn new_document_variants_page(req: Request<Body>) -> ServerResponse {
    let context: &AppContext = req.data().unwrap();

    render_page(
        "pages/new_document_variants_page.html.tera",
        json!({
            "document_types": context.document_types,
        }),
    )
}
