use anyhow::*;
use arhiv_core::Arhiv;
use hyper::{header, Response};
use serde_json::{json, Value};

use crate::templates::TEMPLATES;
use rs_utils::{merge_json, server::ServerResponse};

// see https://doc.rust-lang.org/std/hash/index.html#examples
#[cfg(debug_assertions)]
pub fn get_file_hash(name: impl std::hash::Hash, data: impl std::hash::Hash) -> u64 {
    use std::{collections::hash_map::DefaultHasher, hash::Hasher};

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
        .get_schema()
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

/// Define a function which returns file content from memory
/// in release mode and from file system in debug mode.
macro_rules! embed_file {
    ($name: ident, $rel_file_path: expr) => {
        fn $name() -> std::borrow::Cow<'static, str> {
            use std::borrow::Cow;
            use std::fs;
            use std::path::Path;

            if cfg!(debug_assertions) {
                let source_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file!());
                let source_dir_path = source_file_path
                    .parent()
                    .expect("file must have a parent dir");
                let file_path = source_dir_path
                    .join($rel_file_path)
                    .canonicalize()
                    .expect("failed to canonicalize file path");

                let data = fs::read_to_string(file_path).expect("failed to read file");

                Cow::Owned(data)
            } else {
                Cow::Borrowed(include_str!($rel_file_path))
            }
        }
    };
}
