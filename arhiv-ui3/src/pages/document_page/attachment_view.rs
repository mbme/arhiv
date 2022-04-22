use anyhow::Result;

use arhiv_core::{definitions::Attachment, Arhiv};
use serde_json::json;

use crate::{components::DocumentDataViewer, template_fn, urls::blob_url};

template_fn!(
    render_attachment_view_template,
    "./attachment_view.html.tera"
);

pub fn render_attachment_view(attachment: &Attachment, arhiv: &Arhiv) -> Result<String> {
    let content = DocumentDataViewer::new(attachment).render(arhiv)?;

    render_attachment_view_template(json!({
        "content": content,
        "subtype": attachment.subtype,
        "blob_url": blob_url(&attachment.get_blob_id()),
        "filename": attachment.get_filename(),
    }))
}
