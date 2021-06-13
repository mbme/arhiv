use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use anyhow::*;
use hyper::{header, Response};
use rs_utils::server::ServerResponse;
use serde_json::Value;

use crate::templates::TEMPLATES;

// see https://doc.rust-lang.org/std/hash/index.html#examples
pub fn get_file_hash(name: impl Hash, data: impl Hash) -> u64 {
    let mut s = DefaultHasher::new();

    name.hash(&mut s);
    data.hash(&mut s);

    s.finish()
}

pub fn render_page(template_name: &str, context: Value) -> ServerResponse {
    let result = TEMPLATES.render(template_name, context)?;

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        // prevent page from caching
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::EXPIRES, "0")
        // ---
        .body(result.into())
        .context("failed to build response")
}
