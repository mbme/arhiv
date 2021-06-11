use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{app_context::AppContext, components::Editor, http_utils::AppResponse};
use arhiv::{entities::Document, schema::SCHEMA};

pub async fn new_document_page(req: Request<Body>) -> AppResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let context: &AppContext = req.data().unwrap();

    let data = SCHEMA.create(document_type)?;

    let document = Document::new(document_type.clone(), data.into());

    let editor = Editor::new(&document)?.render(&context)?;

    context.render_page(
        "pages/new_document_page.html.tera",
        json!({
            "editor": editor,
            "document_type": document_type,
        }),
    )
}
