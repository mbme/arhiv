use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{
    entities::{Document, Id},
    Arhiv,
};
use rs_utils::server::{parse_urlencoded, respond_see_other, ServerResponse};

use crate::{urls::document_url, utils::fields_to_document_data};

pub async fn create_document_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let document_type: &str = parts.param("document_type").unwrap();
    let parent_collection: Option<Id> = parts
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = parts.data().unwrap();
    let data_description = arhiv.get_schema().get_data_description(document_type)?;

    let body = hyper::body::to_bytes(body).await?;
    let fields = parse_urlencoded(&body);
    let data = fields_to_document_data(&fields, data_description)?;

    let mut document = Document::new_with_data(document_type, data);
    arhiv.stage_document(&mut document)?;

    respond_see_other(document_url(&document.id, &parent_collection))
}
