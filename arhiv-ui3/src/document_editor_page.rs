use anyhow::*;
use rocket::State;
use rocket_contrib::templates::Template;
use serde_json::json;

use crate::utils::AppContext;

#[get("/documents/<id>/edit")]
pub fn document_editor_page(id: String, context: State<AppContext>) -> Result<Option<Template>> {
    let document = {
        if let Some(document) = context.arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    Ok(Some(Template::render(
        "document_editor_page",
        json!({ "document": document }),
    )))
}
