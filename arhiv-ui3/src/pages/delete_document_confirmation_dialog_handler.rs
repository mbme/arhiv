use anyhow::ensure;
use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::server::{parse_urlencoded, respond_see_other, ServerResponse};

use crate::{pages::get_confirmation_text, urls::parent_collection_url};

pub async fn delete_document_confirmation_dialog_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let id: Id = parts.param("id").unwrap().into();
    let parent_collection: Option<Id> = parts
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = parts.data().unwrap();

    let document = arhiv.must_get_document(&id)?;

    let body = hyper::body::to_bytes(body).await?;
    let fields = parse_urlencoded(&body);

    let confirmation_text = fields
        .get("confirmation_text")
        .map(String::as_str)
        .unwrap_or_default();

    ensure!(
        confirmation_text == get_confirmation_text(&document.document_type),
        "confirmation text is wrong"
    );

    arhiv.delete_document(&id)?;

    respond_see_other(parent_collection_url(
        &document.document_type,
        &parent_collection,
    ))
}
