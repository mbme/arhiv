use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumbs, Editor},
    utils::render_page,
};
use arhiv_core::{entities::*, schema::SCHEMA, Arhiv};
use rs_utils::server::{respond_not_found, ServerResponse};

pub async fn document_editor_page(req: Request<Body>) -> ServerResponse {
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
            .get_data_description_by_type(&document.document_type)?
            .is_internal
    );

    let editor = Editor::new(&document)?.render()?;
    let breadcrumbs = Breadcrumbs::DocumentEditor(&document).render()?;

    render_page(
        "pages/document_editor_page.html.tera",
        json!({
            "breadcrumbs": breadcrumbs,
            "document": document,
            "editor": editor,
        }),
    )
}
