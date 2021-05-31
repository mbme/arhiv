use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::app_context::{AppContext, TemplatePage};

#[catch(404)]
pub fn not_found_page(request: &rocket::Request) -> Result<TemplatePage> {
    let context: State<AppContext> = request.guard().expect("AppContext must be available");

    context.render_page("pages/not_found_page.html.tera", json!({}))
}
