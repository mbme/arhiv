use anyhow::*;
use arhiv_core::Arhiv;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{markup::ArhivMarkupExt, utils::render_page};
use rs_utils::server::ServerResponse;

pub async fn archive_document_confirmation_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let document = arhiv
        .get_document(&id.into())?
        .ok_or(anyhow!("document not found"))?;

    let preview = arhiv.render_preview(&document);

    render_page(
        "pages/archive_document_confirmation_page.html.tera",
        json!({
            "document": document,
            "preview": preview,
        }),
    )
}
