use anyhow::*;
use hyper::StatusCode;
use serde_json::json;

use arhiv_core::{
    definitions::TASK_TYPE,
    entities::{ATTACHMENT_TYPE, TOMBSTONE_TYPE},
    Arhiv,
};
use rs_utils::server::ServerResponse;

use crate::{template_fn, urls::catalog_url, utils::render_content};

template_fn!(render_template, "./base.html.tera");

pub fn render_page_with_status(
    status: StatusCode,
    content: impl AsRef<str>,
    arhiv: &Arhiv,
) -> ServerResponse {
    let result = render_template(json!({
        "sidebar": render_sidebar(arhiv)?,
        "content": content.as_ref(),
    }))?;

    render_content(status, result)
}

pub fn render_page(content: impl AsRef<str>, arhiv: &Arhiv) -> ServerResponse {
    render_page_with_status(StatusCode::OK, content, arhiv)
}

pub fn render_modal(content: impl AsRef<str>) -> ServerResponse {
    let result = render_template(json!({
        "content": content.as_ref(),
    }))?;

    render_content(StatusCode::OK, result)
}

const IGNORED_DOCUMENT_TYPES: &[&str] = &[TOMBSTONE_TYPE, ATTACHMENT_TYPE, TASK_TYPE];

fn get_nav_document_types(arhiv: &Arhiv) -> Vec<(&'static str, String)> {
    arhiv
        .get_schema()
        .get_document_types(false)
        .into_iter()
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .map(|module| (module, catalog_url(module)))
        .collect()
}

template_fn!(render_sidebar_template, "./sidebar.html.tera");

fn render_sidebar(arhiv: &Arhiv) -> Result<String> {
    let nav_document_types = get_nav_document_types(arhiv);

    render_sidebar_template(json!({
        "nav_document_types": nav_document_types,
    }))
}
