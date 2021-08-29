use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Editor, Toolbar},
    pages::base::render_page,
    template_fn,
};
use arhiv_core::{entities::*, Arhiv};
use rs_utils::{
    server::{respond_not_found, RequestQueryExt, ServerResponse},
    QueryBuilder,
};

template_fn!(render_template, "./edit_document_page.html.tera");

pub async fn edit_document_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let id: Id = id.into();

    let arhiv: &Arhiv = req.data().unwrap();

    let document = {
        if let Some(document) = arhiv.get_document(&id)? {
            document
        } else {
            return respond_not_found();
        }
    };

    // deny editing internal types
    ensure!(
        !arhiv
            .get_schema()
            .get_data_description(&document.document_type)?
            .is_internal
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
        .with_breadcrumb(Breadcrumb::Document(&document))
        .with_breadcrumb(Breadcrumb::String("editor".to_string()))
        .on_close_document(&document)
        .render(arhiv)?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "document": document,
        "editor": editor,
    }))?;

    render_page(content, arhiv)
}
