use anyhow::Context;
use hyper::{header, Response, StatusCode};

use rs_utils::http_server::ServerResponse;

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
