use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    app_context::AppContext,
    components::Editor,
    http_utils::{not_found, AppResponse},
};
use arhiv_core::entities::*;

pub async fn document_editor_page(req: Request<Body>) -> AppResponse {
    let id: &str = req.param("id").unwrap();
    let id: Id = id.into();

    let context: &AppContext = req.data().unwrap();

    let document = {
        if let Some(document) = context.arhiv.get_document(&id)? {
            document
        } else {
            return not_found();
        }
    };

    let editor = Editor::new(&document)?.render(&context)?;

    context.render_page(
        "pages/document_editor_page.html.tera",
        json!({
            "document": document, //
            "editor": editor,
        }),
    )
}
