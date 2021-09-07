use anyhow::*;
use hyper::{header, Response};
use rs_utils::server::ServerResponse;
use serde_json::json;

use arhiv_core::Arhiv;

use crate::{template_fn, urls::catalog_url};

template_fn!(render_template, "./base.html.tera");

pub fn render_page(content: impl AsRef<str>, arhiv: &Arhiv) -> ServerResponse {
    let nav_document_types = get_nav_document_types(arhiv);

    let result = render_template(json!({
        "nav_document_types": nav_document_types,
        "content": content.as_ref(),
    }))?;

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        // prevent page from caching
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::EXPIRES, "0")
        // ---
        .body(result.into())
        .context("failed to build response")
}

const IGNORED_DOCUMENT_TYPES: &[&str] = &["tombstone", "attachment", "task"];

fn get_nav_document_types(arhiv: &Arhiv) -> Vec<(&'static str, String)> {
    arhiv
        .get_schema()
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .map(|module| (module, catalog_url(module)))
        .collect()
}
