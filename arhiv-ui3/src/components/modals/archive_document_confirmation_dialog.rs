use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv};

use crate::{components::modals::modal::render_modal, template_fn};

template_fn!(
    render_template,
    "./archive_document_confirmation_dialog.html.tera"
);

pub fn render_archive_document_confirmation_dialog(id: &Id, arhiv: &Arhiv) -> Result<String> {
    let document = arhiv
        .get_document(id)?
        .ok_or_else(|| anyhow!("document not found"))?;

    let document_title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document": document,
        "title": document_title,
    }))?;

    let modal_title = if document.archived {
        format!("UnArchive {}", &document.document_type)
    } else {
        format!("Archive {}", &document.document_type)
    };

    render_modal(
        "archive-document-confirmation-dialog",
        &modal_title,
        &content,
    )
}
