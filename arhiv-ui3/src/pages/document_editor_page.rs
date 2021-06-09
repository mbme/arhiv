use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::{
    app_context::{AppContext, TemplatePage},
    components::Editor,
};
use arhiv::entities::*;

#[get("/documents/<id>/edit")]
pub fn document_editor_page(
    id: String,
    context: State<AppContext>,
) -> Result<Option<TemplatePage>> {
    let id: Id = id.into();

    let document = {
        if let Some(document) = context.arhiv.get_document(&id)? {
            document
        } else {
            return Ok(None);
        }
    };

    let editor = Editor::new(&document)?.render(&context)?;

    Ok(Some(context.render_page(
        "pages/document_editor_page.html.tera",
        json!({
            "document": document, //
            "editor": editor,
        }),
    )?))
}
