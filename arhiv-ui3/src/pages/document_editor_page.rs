use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::app_context::{AppContext, TemplatePage};

#[get("/documents/<id>/edit")]
pub fn document_editor_page(
    id: String,
    context: State<AppContext>,
) -> Result<Option<TemplatePage>> {
    let document = {
        if let Some(document) = context.arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    Ok(Some(context.render_page(
        "pages/document_editor_page.html.tera",
        json!({ "document": document }),
    )?))
}
