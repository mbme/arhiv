use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::server::ServerResponse;

use crate::{
    pages::base::render_modal,
    template_fn,
    urls::{document_url, parent_collection_url},
};

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

    let document_title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document": document,
        "title": document_title,
        "cancel_url": document_url(&document.id, &collection_id),
        "confirm_url": parent_collection_url(&document.document_type, &collection_id),
    }))?;

    render_modal(content)
}
