use anyhow::*;
use serde_json::json;

use arhiv_core::{entities::Id, Arhiv};

use crate::{components::modals::modal::render_modal, template_fn, urls::parent_collection_url};

template_fn!(
    render_template,
    "./delete_document_confirmation_dialog.html.tera"
);

pub fn render_delete_document_confirmation_dialog(
    id: &Id,
    parent_collection: &Option<Id>,
    arhiv: &Arhiv,
) -> Result<String> {
    let document = arhiv
        .get_document(id)?
        .ok_or_else(|| anyhow!("document not found"))?;

    let document_title = arhiv.get_schema().get_title(&document)?;

    let content = render_template(json!({
        "document": document,
        "title": document_title,
        "confirm_url": parent_collection_url(&document.document_type, parent_collection),
    }))?;

    let modal_title = format!("Delete {}", &document.document_type);

    render_modal(
        "delete-document-confirmation-dialog",
        &modal_title,
        &content,
    )
}
