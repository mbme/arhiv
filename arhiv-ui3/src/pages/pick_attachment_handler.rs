use anyhow::*;
use hyper::{Body, Request, StatusCode};
use routerify::ext::RequestExt;

use arhiv_core::Arhiv;
use rs_utils::{run_command, server::ServerResponse};

use crate::utils::render_content;

pub async fn pick_attachment_handler(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let files = run_command("mb-filepicker", vec![])?;
    let files: Vec<String> = serde_json::from_str(&files)?;
    ensure!(files.len() < 2, "must not select multiple files");

    let mut response = String::new();

    if let Some(file_path) = files.get(0) {
        let document = arhiv.add_attachment(file_path, false)?;
        response = document.id.to_string();
    }

    render_content(StatusCode::OK, response)
}
