use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::server::ServerResponse;

use super::render_modal;
use crate::{template_fn, urls::parent_collection_url};

template_fn!(
    render_template,
    "./delete_document_confirmation_dialog.html.tera"
);

pub async fn render_delete_document_confirmation_dialog(req: Request<Body>) -> ServerResponse {
    let id: Id = req.param("id").unwrap().into();
    let collection_id: Option<Id> = req
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = req.data().unwrap();
    let document = arhiv
        .get_document(id)?
        .ok_or_else(|| anyhow!("document not found"))?;

    let document_title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document": document,
        "title": document_title,
        "confirm_url": parent_collection_url(&document.document_type, &collection_id),
    }))?;

    let modal_title = format!("Delete {}", &document.document_type);

    render_modal(
        "delete-document-confirmation-dialog",
        &modal_title,
        &content,
        true,
    )
}
