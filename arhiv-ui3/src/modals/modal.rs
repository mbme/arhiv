use serde_json::json;

use rs_utils::server::ServerResponse;

use crate::{template_fn, utils::render_content};

template_fn!(render_template, "./modal.html.tera");

pub fn render_modal(
    dialog_id: &str,
    title: &str,
    content: &str,
    with_spacer: bool,
) -> ServerResponse {
    let content = render_template(json!({
        "dialog_id": dialog_id,
        "title": title,
        "with_spacer": with_spacer,
        "content": content,
    }))?;

    render_content(content)
}
