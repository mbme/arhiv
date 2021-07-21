use anyhow::ensure;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Editor, Toolbar},
    utils::ArhivPageExt,
};
use arhiv_core::{entities::Document, Arhiv};
use rs_utils::server::{RequestQueryExt, ServerResponse};

pub async fn new_document_page(req: Request<Body>) -> ServerResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let arhiv: &Arhiv = req.data().unwrap();

    let data_description = arhiv.schema.get_data_description(document_type)?;

    ensure!(!data_description.is_internal);

    let params = req.get_query_params();

    let document = Document::new_with_data(document_type.clone(), params.into());

    let editor = Editor::new(
        &document,
        arhiv.schema.get_data_description(&document.document_type)?,
    )?
    .render()?;

    let toolbar = Toolbar::new(req.get_query_param("parent_collection"))
        .with_breadcrubs(vec![
            Breadcrumb::Collection(document.document_type.to_string()),
            Breadcrumb::String(format!("new {}", document.document_type)),
        ])
        .on_close_document(&document)
        .render()?;

    arhiv.render_page(
        "pages/new_document_page.html.tera",
        json!({
            "toolbar": toolbar,
            "editor": editor,
            "document_type": document_type,
        }),
    )
}
