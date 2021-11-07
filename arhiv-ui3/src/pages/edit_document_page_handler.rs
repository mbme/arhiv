use hyper::{http::request::Parts, Body, Request, StatusCode};
use routerify::ext::RequestExt;

use arhiv_core::{entities::Id, Arhiv, Validator};
use rs_utils::server::{parse_urlencoded, respond_see_other, ServerResponse};

use super::{base::render_page_with_status, render_edit_document_page_content};
use crate::{urls::document_url, utils::fields_to_document_data};

pub async fn edit_document_page_handler(req: Request<Body>) -> ServerResponse {
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

    let prev_data = document.data;
    document.data = fields_to_document_data(&fields, data_description)?;

    let validation_result =
        Validator::new(arhiv).validate(&document.data, Some(&prev_data), data_description);

    if let Err(error) = validation_result {
        let content = render_edit_document_page_content(
            &document,
            &Some(error.errors),
            &parent_collection,
            arhiv,
        )?;

        return render_page_with_status(StatusCode::UNPROCESSABLE_ENTITY, content, arhiv);
    }

    arhiv.stage_document(&mut document)?;

    respond_see_other(document_url(&id, &parent_collection))
}
