use anyhow::*;
use rocket::State;
use rocket_contrib::templates::Template;
use serde_json::json;

use arhiv::Arhiv;

#[get("/documents/<id>")]
pub fn document_page(id: String, arhiv: State<Arhiv>) -> Result<Option<Template>> {
    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    Ok(Some(Template::render(
        "document_page",
        json!({ "document": document }),
    )))
}
