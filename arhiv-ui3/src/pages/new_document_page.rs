use anyhow::*;
use rocket::State;
use serde_json::json;

use crate::{
    app_context::{AppContext, TemplatePage},
    components::Editor,
};

#[get("/new?<document_type>")]
pub fn new_document_page(
    document_type: Option<String>,
    context: State<AppContext>,
) -> Result<TemplatePage> {
    let editor = if let Some(ref document_type) = document_type {
        Editor::new(
            document_type, //
            &json!({}),
            Some("/".to_string()),
        )?
        .render(&context)?
    } else {
        "".to_string()
    };

    Ok(context.render_page(
        "pages/new_document_page.html.tera",
        json!({
            "editor": editor,
            "document_types": context.document_types,
        }),
    )?)
}
