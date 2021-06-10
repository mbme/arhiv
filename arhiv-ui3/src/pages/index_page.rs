use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{app_context::AppContext, http_utils::AppResponse};

pub async fn index_page(req: Request<Body>) -> AppResponse {
    let context: &AppContext = req.data().unwrap();

    let status = context.arhiv.get_status()?;

    context.render_page(
        "pages/index_page.html.tera",
        json!({
            "status": status.to_string(),
            "document_types": context.document_types,
        }),
    )
}
