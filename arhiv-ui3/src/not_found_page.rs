use rocket_contrib::templates::Template;
use serde_json::json;

#[catch(404)]
pub fn not_found_page(_request: &rocket::Request) -> Template {
    Template::render("not_found_page", json!({}))
}
