use anyhow::*;
use askama::Template;
use rocket::State;

use arhiv::{entities::Document, Arhiv};

use crate::utils::TemplateContext;

#[derive(Template)]
#[template(path = "document_page.html")]
pub struct DocumentPage {
    context: TemplateContext,
    document: Document,
}

#[get("/documents/<id>")]
pub fn render_document_page(
    id: String,
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<Option<DocumentPage>> {
    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    Ok(Some(DocumentPage {
        context: context.clone(),
        document,
    }))
}
