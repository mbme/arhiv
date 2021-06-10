use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{app_context::AppContext, components::Editor, http_utils::AppResponse};
use arhiv::{entities::Document, schema::SCHEMA};

pub async fn new_document_page(req: Request<Body>) -> AppResponse {
    let document_type = req.param("document_type");
    let context: &AppContext = req.data().unwrap();

    let editor = if let Some(document_type) = document_type {
        let data = SCHEMA.create(document_type.clone())?;

        let document = Document::new(document_type.clone(), data.into());

        Editor::new(&document)?.render(&context)?
    } else {
        "".to_string()
    };

    context.render_page(
        "pages/new_document_page.html.tera",
        json!({
            "editor": editor,
            "document_type": document_type,
            "document_types": context.document_types,
        }),
    )
}
