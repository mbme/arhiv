use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::{
    app_context::{AppContext, TemplatePage},
    components::Editor,
};
use arhiv::{entities::Document, schema::SCHEMA};

#[get("/new?<document_type>")]
pub fn new_document_page(
    document_type: Option<String>,
    context: State<AppContext>,
) -> Result<TemplatePage> {
    let editor = if let Some(ref document_type) = document_type {
        let data = SCHEMA.create(document_type.clone())?;

        let document = Document::new(document_type.clone(), data.into());

        Editor::new(&document)?.render(&context)?
    } else {
        "".to_string()
    };

    Ok(context.render_page(
        "pages/new_document_page.html.tera",
        json!({
            "editor": editor,
            "document_type": document_type,
            "document_types": context.document_types,
        }),
    )?)
}
