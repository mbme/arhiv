use anyhow::Result;
use rocket::State;
use rocket_contrib::templates::Template;
use serde_json::json;

use arhiv::schema::SCHEMA;

use crate::utils::AppContext;

#[get("/")]
pub fn index_page(context: State<AppContext>) -> Result<Template> {
    let status = context.arhiv.get_status()?;

    let document_types: Vec<&str> = SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect();

    Ok(Template::render(
        "pages/index_page",
        json!({
            "status": status.to_string(),
            "document_types": document_types,
        }),
    ))
}
