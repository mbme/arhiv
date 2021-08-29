use std::collections::HashMap;

use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Editor, Toolbar},
    pages::base::render_page,
    template_fn,
};
use arhiv_core::{
    entities::{Document, DocumentData},
    schema::DataDescription,
    Arhiv,
};
use rs_utils::{
    server::{RequestQueryExt, ServerResponse},
    QueryBuilder,
};

template_fn!(render_template, "./new_document_page.html.tera");

pub async fn new_document_page(req: Request<Body>) -> ServerResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let arhiv: &Arhiv = req.data().unwrap();

    let data_description = arhiv.get_schema().get_data_description(document_type)?;

    ensure!(!data_description.is_internal);

    let params = req.get_query_params();

    let document = Document::new_with_data(
        document_type.clone(),
        params_to_document_data(params, data_description)?,
    );

    let editor = Editor::new(
        &document,
        arhiv
            .get_schema()
            .get_data_description(&document.document_type)?,
    )?
    .with_document_query(
        QueryBuilder::new()
            .maybe_add_param(
                "parent_collection",
                req.get_query_param("parent_collection"),
            )
            .build(),
    )
    .render()?;

    let toolbar = Toolbar::new(req.get_query_param("parent_collection"))
        .with_breadcrumb(Breadcrumb::Collection(document.document_type.to_string()))
        .with_breadcrumb(Breadcrumb::String(format!(
            "new {}",
            document.document_type
        )))
        .on_close_document(&document)
        .render(arhiv)?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "editor": editor,
        "document_type": document_type,
    }))?;

    render_page(content, arhiv)
}

fn params_to_document_data(
    params: HashMap<String, String>,
    data_description: &DataDescription,
) -> Result<DocumentData> {
    let mut data = DocumentData::new();

    for ref field in &data_description.fields {
        let raw_value = if let Some(value) = params.get(field.name) {
            value
        } else {
            continue;
        };

        let value = field.from_string(raw_value)?;
        data.set(field.name, value);
    }

    Ok(data)
}
