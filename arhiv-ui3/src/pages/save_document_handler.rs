use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::server::{parse_urlencoded, respond_see_other, ServerResponse};

use crate::{urls::document_url, utils::fields_to_document_data};

pub async fn save_document_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let id: Id = parts.param("id").unwrap().into();
    let parent_collection: Option<Id> = parts
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = parts.data().unwrap();

    let mut document = arhiv.must_get_document(&id)?;

    let data_description = arhiv
        .get_schema()
        .get_data_description(&document.document_type)?;

    let body = hyper::body::to_bytes(body).await?;
    let fields = parse_urlencoded(&body);

    document.data = fields_to_document_data(&fields, data_description)?;
    arhiv.stage_document(&mut document)?;

    respond_see_other(document_url(&id, &parent_collection))
}
