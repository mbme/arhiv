use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{app_context::AppContext, utils::render_page};
use rs_utils::server::ServerResponse;

pub async fn index_page(req: Request<Body>) -> ServerResponse {
    let context: &AppContext = req.data().unwrap();

    let status = context.arhiv.get_status()?;

    render_page(
        "pages/index_page.html.tera",
        json!({
            "status": status.to_string(),
            "document_types": context.document_types,
        }),
    )
}
