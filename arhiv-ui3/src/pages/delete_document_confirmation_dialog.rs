use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::server::{respond_moved_permanently, ServerResponse};

use crate::{pages::base::render_modal, template_fn, urls::document_url};

template_fn!(
    render_template,
    "./delete_document_confirmation_dialog.html.tera"
);

pub async fn delete_document_confirmation_dialog(req: Request<Body>) -> ServerResponse {
    let id: Id = req.param("id").unwrap().into();
    let collection_id: Option<Id> = req
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = req.data().unwrap();
    let document = arhiv
        .get_document(id)?
        .ok_or_else(|| anyhow!("document not found"))?;

    if document.is_tombstone() {
        return respond_moved_permanently(document_url(&document.id, &None));
    }

    let document_title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document_type": document.document_type,
        "title": document_title,
        "confirmation_text": get_confirmation_text(&document.document_type),
        "cancel_url": document_url(&document.id, &collection_id),
    }))?;

    render_modal(content)
}

pub fn get_confirmation_text(document_type: &str) -> String {
    format!("delete {}", document_type)
}
