use askama::Template;

use arhiv::schema::SCHEMA;
use rocket::State;

use crate::utils::TemplateContext;

#[derive(Template)]
#[template(path = "catalog_index_page.html")]
pub struct CatalogIndexPage {
    context: TemplateContext,
    document_types: Vec<&'static str>,
}

#[get("/catalogs")]
pub fn render_catalog_index_page(context: State<TemplateContext>) -> CatalogIndexPage {
    CatalogIndexPage {
        context: context.clone(),
        document_types: SCHEMA
            .modules
            .iter()
            .map(|module| module.document_type)
            .collect(),
    }
}
