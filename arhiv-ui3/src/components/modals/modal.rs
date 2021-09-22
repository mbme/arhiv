use anyhow::*;
use serde_json::json;

use crate::template_fn;

template_fn!(render_template, "./modal.html.tera");

pub fn render_modal(dialog_id: &str, title: &str, content: &str) -> Result<String> {
    render_template(json!({
        "dialog_id": dialog_id,
        "title": title,
        "content": content,
    }))
}
