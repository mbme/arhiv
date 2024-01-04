use std::str::FromStr;

use anyhow::Context;
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{self, HeaderMapExt},
    TypedHeader,
};
use rust_embed::RustEmbed;

use rs_utils::{bytes_to_hex_string, get_mime_from_path, http_server::ServerError};

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/public"]
struct PublicAssets;

pub async fn public_assets_handler(
    Path(asset): Path<String>,
    if_none_match: Option<TypedHeader<headers::IfNoneMatch>>,
) -> Result<Response, ServerError> {
    let embedded_file = {
        if let Some(data) = PublicAssets::get(&asset) {
            data
        } else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }
    };

    let hash = embedded_file.metadata.sha256_hash();
    let hash = bytes_to_hex_string(&hash);

    let etag = headers::ETag::from_str(&format!("\"{hash}\"")).context("failed to parse ETag")?;

    if let Some(if_none_match) = if_none_match {
        // TODO ensure it works
        if !if_none_match.precondition_passes(&etag) {
            return Ok(StatusCode::NOT_MODIFIED.into_response());
        }
    }

    let mime = get_mime_from_path(asset);

    let mut headers = HeaderMap::new();
    headers.typed_insert(etag);
    headers.typed_insert(headers::ContentType::from_str(&mime)?);

    Ok((StatusCode::OK, headers, embedded_file.data).into_response())
}
