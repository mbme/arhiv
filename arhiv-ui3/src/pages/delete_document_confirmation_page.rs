use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::app_context::{AppContext, TemplatePage};

#[get("/documents/<id>/delete")]
pub fn delete_document_confirmation_page(
    id: String,
    context: State<AppContext>,
) -> Result<TemplatePage> {
    let document = context
        .arhiv
        .get_document(&id.into())?
        .ok_or(anyhow!("document not found"))?;

    ensure!(!document.is_tombstone(), "document already deleted");

    let preview = context.render_preview(&document);

    context.render_page(
        "pages/delete_document_confirmation_page.html.tera",
        json!({
            "document": document,
            "preview": preview,
        }),
    )
}
