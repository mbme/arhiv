use std::fs;

use anyhow::*;
use hyper::{Body, Request, StatusCode};
use serde_json::json;

use rs_utils::{
    ensure_file_exists,
    server::{RequestQueryExt, ServerResponse},
};

use crate::{template_fn, urls::pick_file_confirmation_handler_url, utils::render_content};

template_fn!(render_template, "./pick_file_confirmation_modal.html.tera");

pub async fn pick_file_confirmation_modal(req: Request<Body>) -> ServerResponse {
    let url = req.get_url();

    let file_path = url
        .get_query_param("file")
        .ok_or_else(|| anyhow!("file query param must be present"))?;

    ensure_file_exists(file_path)?;

    let metadata = fs::metadata(file_path)?;
    let size = metadata.len();

    let content = render_template(json!({
        "handler_url": pick_file_confirmation_handler_url(),
        "file_path": file_path,
        "size": size,
    }))?;

    render_content(StatusCode::OK, content)
}
