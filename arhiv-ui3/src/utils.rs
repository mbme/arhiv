use std::collections::HashMap;

use anyhow::*;
use hyper::{header, Response, StatusCode};

use arhiv_core::{entities::DocumentData, schema::DataDescription};
use rs_utils::server::ServerResponse;

/// `template_fn!(pub get_markup, "./markup.rs");`
#[macro_export]
macro_rules! template_fn {
    ($vis:vis $name:ident, $rel_file_path:expr) => {
        fn $name(context: impl serde::Serialize) -> anyhow::Result<String> {
            use anyhow::Context;
            use std::fs;
            use std::path::Path;
            use tera::{Context as TeraContext, Tera};

            let context = TeraContext::from_value(
                serde_json::to_value(context).context("failed to serialize context")?,
            )
            .context("failed to create context")?;

            if cfg!(debug_assertions) {
                let source_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("..") // FIXME https://github.com/rust-lang/cargo/issues/3946
                    .join(file!());
                let source_dir_path = source_file_path
                    .parent()
                    .context("file must have a parent dir")?;
                let file_path = source_dir_path
                    .join($rel_file_path)
                    .canonicalize()
                    .context("failed to canonicalize file path")?;

                let data = fs::read_to_string(file_path).context("failed to read file")?;

                Tera::one_off(&data, &context, true).context("failed to render template")
            } else {
                let data = include_str!($rel_file_path);

                Tera::one_off(data, &context, true).context("failed to render template")
            }
        }
    };
}

pub fn render_content(status: StatusCode, content: String) -> ServerResponse {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html")
        // prevent page from caching
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::EXPIRES, "0")
        // ---
        .body(content.into())
        .context("failed to build response")
}

pub fn fields_to_document_data(
    fields: &HashMap<String, String>,
    data_description: &DataDescription,
) -> Result<DocumentData> {
    let mut data = DocumentData::new();

    for field in &data_description.fields {
        let raw_value = if let Some(value) = fields.get(field.name) {
            value
        } else {
            continue;
        };

        let value = field.from_string(raw_value)?;
        data.set(field.name, value);
    }

    Ok(data)
}
