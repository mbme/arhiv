use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::app_context::AppContext;
use rs_utils::server::ServerResponse;

pub async fn delete_document_confirmation_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let context: &AppContext = req.data().unwrap();

    let document = context
        .arhiv
        .get_document(&id.into())?
        .ok_or(anyhow!("document not found"))?;

    ensure!(!document.is_tombstone(), "document already deleted");

    let preview = context.render_preview(&document);

    context.render_page(
        "pages/delete_document_confirmation_page.html.tera",
        json!({
            "document": document,
            "preview": preview,
        }),
    )
}
