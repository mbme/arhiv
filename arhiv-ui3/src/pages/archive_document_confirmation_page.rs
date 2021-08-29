use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{pages::base::render_page, template_fn};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

template_fn!(
    render_template,
    "./archive_document_confirmation_page.html.tera"
);

pub async fn archive_document_confirmation_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let document = arhiv
        .get_document(id)?
        .ok_or(anyhow!("document not found"))?;

    let title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document": document,
        "title": title,
    }))?;

    render_page(content, arhiv)
}
