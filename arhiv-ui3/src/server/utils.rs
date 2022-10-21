use anyhow::Context;
use hyper::{header, Response, StatusCode};

use rs_utils::http_server::ServerResponse;

/// `include_dynamic_str!("./markup.rs");`
#[macro_export]
macro_rules! include_dynamic_str {
    ($rel_file_path:expr) => {{
        use std::borrow::Cow;
        use std::fs;
        use std::path::Path;

        use anyhow::Context;

        let result: Result<Cow<'static, str>> = if cfg!(debug_assertions) {
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

            Ok(Cow::Owned(data))
        } else {
            let data = include_str!($rel_file_path);

            Ok(Cow::Borrowed(data))
        };

        result
    }};
}

fn build_response(status: StatusCode, content_type: &str, content: String) -> ServerResponse {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        // prevent page from caching
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::EXPIRES, "0")
        // ---
        .body(content.into())
        .context("failed to build response")
}

pub fn render_content(status: StatusCode, content: String) -> ServerResponse {
    build_response(status, "text/html", content)
}

pub fn render_json(status: StatusCode, content: String) -> ServerResponse {
    build_response(status, "application/json", content)
}
