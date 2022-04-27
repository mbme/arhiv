use anyhow::Result;

use arhiv_core::{definitions::Attachment, entities::Document, Arhiv};
use serde_json::json;

use crate::{components::DocumentDataViewer, template_fn, urls::blob_url};

template_fn!(
    render_attachment_view_template,
    "./attachment_view.html.tera"
);

pub fn render_attachment_view(document: Document, arhiv: &Arhiv) -> Result<String> {
    let content = DocumentDataViewer::new(&document).render(arhiv)?;

    let attachment: Attachment = document.try_into()?;

    render_attachment_view_template(json!({
        "content": content,
        "subtype": attachment.subtype,
        "blob_url": blob_url(&attachment.data.blob),
        "filename": attachment.data.filename,
    }))
}
