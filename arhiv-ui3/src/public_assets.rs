use anyhow::*;
use hyper::{header, Body, Request, Response, StatusCode};
use routerify::ext::RequestExt;
use rust_embed::RustEmbed;

use rs_utils::{
    get_mime_from_path,
    server::{respond_not_found, respond_with_status, ServerResponse},
};

use crate::utils::get_file_hash;

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

    let hash = get_file_hash(asset, &data).to_string();

    if let Some(etag) = req.headers().get(header::IF_NONE_MATCH) {
        if etag.to_str()? == hash {
            return respond_with_status(StatusCode::NOT_MODIFIED);
        }
    }

    let mime = get_mime_from_path(&asset);

    Response::builder()
        .header(header::ETAG, hash)
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(data))
        .context("failed to build response")
}
