use anyhow::Result;
use rocket::State;
use rocket_contrib::templates::Template;
use serde_json::json;

use arhiv::Arhiv;

#[get("/")]
pub fn index_page(arhiv: State<Arhiv>) -> Result<Template> {
    let status = &arhiv.get_status()?;

    Ok(Template::render(
        "index_page",
        json!({
            "status": status.to_string(),
        }),
    ))
}
