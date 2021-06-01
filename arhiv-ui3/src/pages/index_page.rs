use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::app_context::{AppContext, TemplatePage};

#[get("/")]
pub fn index_page(context: State<AppContext>) -> Result<TemplatePage> {
    let status = context.arhiv.get_status()?;

    context.render_page(
        "pages/index_page.html.tera",
        json!({
            "status": status.to_string(),
            "document_types": context.document_types,
        }),
    )
}
