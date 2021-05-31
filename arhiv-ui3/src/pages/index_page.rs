use anyhow::*;
use rocket::State;
use serde_json::json;

use arhiv::schema::SCHEMA;

use crate::app_context::{AppContext, TemplatePage};

#[get("/")]
pub fn index_page(context: State<AppContext>) -> Result<TemplatePage> {
    let status = context.arhiv.get_status()?;

    let document_types: Vec<&str> = SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect();

    context.render_page(
        "pages/index_page.html.tera",
        json!({
            "status": status.to_string(),
            "document_types": document_types,
        }),
    )
}
