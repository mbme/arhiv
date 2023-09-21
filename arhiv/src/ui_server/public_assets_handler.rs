use anyhow::Context;
use hyper::{header, Body, Request, Response, StatusCode};
use routerify::ext::RequestExt;
use rust_embed::RustEmbed;

use rs_utils::{
    bytes_to_hex_string, get_mime_from_path,
    http_server::{respond_not_found, respond_with_status, ServerResponse},
};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/public"]
struct PublicAssets;

pub async fn public_assets_handler(req: Request<Body>) -> ServerResponse {
    let asset = req.param("fileName").unwrap();

    let embedded_file = {
        if let Some(data) = PublicAssets::get(asset) {
            data
        } else {
            return respond_not_found();
        }
    };

    let hash = embedded_file.metadata.sha256_hash();
    let hash = bytes_to_hex_string(&hash);

    if let Some(etag) = req.headers().get(header::IF_NONE_MATCH) {
        if etag.to_str()? == hash {
            return respond_with_status(StatusCode::NOT_MODIFIED);
        }
    }

    let mime = get_mime_from_path(asset);

    Response::builder()
        .header(header::ETAG, hash)
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(embedded_file.data))
        .context("failed to build response")
}
