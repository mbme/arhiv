use anyhow::*;
use hyper::{header, Body, Request, Response};
use routerify::ext::RequestExt;
use rust_embed::RustEmbed;

use rs_utils::server::{respond_not_found, ServerResponse};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/public"]
struct PublicAssets;

pub async fn public_assets_handler(req: Request<Body>) -> ServerResponse {
    let asset = req.param("fileName").unwrap();

    let data: Vec<u8> = {
        if let Some(data) = PublicAssets::get(&asset) {
            data.into()
        } else {
            return respond_not_found();
        }
    };

    let mime: String = mime_guess::from_path(&asset)
        .first_or_octet_stream()
        .to_string();

    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(data))
        .context("failed to build response")
}
