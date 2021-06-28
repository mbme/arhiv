use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Editor, Toolbar},
    utils::render_page,
};
use arhiv_core::{entities::*, schema::SCHEMA, Arhiv};
use rs_utils::server::{respond_not_found, ServerResponse};

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

    ensure!(
        !SCHEMA
            .get_data_description(&document.document_type)?
            .is_internal
    );

    let editor = Editor::new(&document)?.render()?;

    let toolbar = Toolbar::new()
        .with_breadcrubs(vec![
            Breadcrumb::for_document_collection(&document)?,
            Breadcrumb::for_document(&document, true),
            Breadcrumb::for_string("editor"),
        ])
        .on_close(format!("/documents/{}", &document.id))
        .render()?;

    render_page(
        "pages/edit_document_page.html.tera",
        json!({
            "toolbar": toolbar,
            "document": document,
            "editor": editor,
        }),
    )
}
