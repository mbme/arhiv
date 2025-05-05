use std::str::FromStr;

use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::headers::{self, HeaderMapExt};

use rs_utils::{get_mime_from_path, http_server::ServerError};

#[cfg(any(feature = "embed-public", not(debug_assertions)))]
fn get_public_file(rel_file_path: &str) -> Option<&'static [u8]> {
    use include_dir::{Dir, include_dir};

    static PUBLIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/public");

    let file = PUBLIC_DIR.get_file(rel_file_path)?;

    Some(file.contents())
}

#[cfg(all(not(feature = "embed-public"), debug_assertions))]
fn get_public_file(rel_file_path: &str) -> Option<Vec<u8>> {
    let path = format!("{}/public/{}", env!("CARGO_MANIFEST_DIR"), rel_file_path);

    std::fs::read(path).ok()
}

pub async fn public_assets_handler(Path(asset): Path<String>) -> Result<Response, ServerError> {
    let data = if let Some(data) = get_public_file(&asset) {
        data
    } else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mime = get_mime_from_path(asset);

    let mut headers = HeaderMap::new();
    headers.typed_insert(headers::ContentType::from_str(&mime)?);

    Ok((StatusCode::OK, headers, data).into_response())
}
