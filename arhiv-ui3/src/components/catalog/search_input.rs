use anyhow::Result;
use serde_json::json;

use crate::template_fn;

template_fn!(render_template, "./search_input.html.tera");

pub fn render_search_input(
    pattern: &str,
    document_type: Option<&str>,
    url: &str,
) -> Result<String> {
    let placeholder = format!("Search {}s", document_type.unwrap_or("document"));

    render_template(json!({
        "pattern": pattern,
        "placeholder": placeholder,
        "url": url,
    }))
}
