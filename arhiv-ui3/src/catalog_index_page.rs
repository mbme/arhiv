use rocket_contrib::templates::Template;
use serde_json::json;

use arhiv::schema::SCHEMA;

#[get("/catalogs")]
pub fn catalog_index_page() -> Template {
    let document_types: Vec<&str> = SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect();

    Template::render(
        "catalog_index_page",
        json!({
            "document_types": document_types,
        }),
    )
}
