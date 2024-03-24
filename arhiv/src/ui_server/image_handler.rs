use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::{self, HeaderMapExt};
use serde::Deserialize;

use baza::entities::BLOBId;
use rs_utils::{
    http_server::{add_max_cache_header, ServerError},
    image::scale_image_async,
};

use super::UIState;

#[derive(Deserialize, Debug)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

#[tracing::instrument(skip(state), level = "debug")]
pub async fn image_handler(
    state: State<Arc<UIState>>,
    Path(blob_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let arhiv = state.must_get_arhiv()?;

    let blob_id = BLOBId::from_string(blob_id);

    let blob = arhiv
        .baza
        .get_connection()?
        .get_existing_blob(&blob_id)?
        .context("BLOB is missing")?;

    let original_size = blob.get_size()?;

    let body = scale_image_async(&blob.file_path, params.max_w, params.max_h)
        .await
        .context("failed to scale image")?;

    let scaled_img_size = body.len();

    let scaled_img_size_percent = scaled_img_size as u64 * 100 / original_size;

    tracing::info!(
        "scaled image from {original_size} bytes to {scaled_img_size} bytes: to {scaled_img_size_percent}%",
    );

    let mut headers = HeaderMap::new();
    add_max_cache_header(&mut headers);
    headers.typed_insert(headers::ContentType::from_str("image/webp")?);
    headers.typed_insert(headers::ContentLength(body.len() as u64));

    Ok((StatusCode::OK, headers, body))
}
