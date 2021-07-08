use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use anyhow::*;
use arhiv_core::Arhiv;
use hyper::{header, Response};
use serde_json::{json, Value};

use crate::templates::TEMPLATES;
use rs_utils::{merge_json, server::ServerResponse};

// see https://doc.rust-lang.org/std/hash/index.html#examples
pub fn get_file_hash(name: impl Hash, data: impl Hash) -> u64 {
    let mut s = DefaultHasher::new();

    name.hash(&mut s);
    data.hash(&mut s);

    s.finish()
}

pub trait ArhivPageExt {
    fn render_page(&self, template_name: &str, context: Value) -> ServerResponse;
}

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

fn get_global_context(arhiv: &Arhiv) -> Value {
    let nav_document_types: Vec<_> = arhiv
        .schema
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .collect();

    json!({
        "global": json!({
            "nav_document_types": nav_document_types,
        }),
    })
}

impl ArhivPageExt for Arhiv {
    fn render_page(&self, template_name: &str, context: Value) -> ServerResponse {
        let global_context = get_global_context(&self);

        let result = TEMPLATES.render(template_name, merge_json(global_context, context)?)?;

        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            // prevent page from caching
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .header(header::EXPIRES, "0")
            // ---
            .body(result.into())
            .context("failed to build response")
    }
}
