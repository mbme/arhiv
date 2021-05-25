use anyhow::*;
use askama::Template;
use rocket::State;

use arhiv::{entities::Document, Arhiv};

use crate::utils::TemplateContext;

#[derive(Template)]
#[template(path = "document_editor_page.html")]
pub struct DocumentEditorPage {
    context: TemplateContext,
    document: Document,
}

#[get("/documents/<id>/edit")]
pub fn render_document_editor_page(
    id: String,
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<Option<DocumentEditorPage>> {
    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    Ok(Some(DocumentEditorPage {
        context: context.clone(),
        document,
    }))
}
