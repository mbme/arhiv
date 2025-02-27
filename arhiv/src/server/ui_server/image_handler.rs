use std::{io::BufReader, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::{self, HeaderMapExt};
use serde::Deserialize;

use baza::{entities::BLOBId, schema::get_asset_by_blob_id};
use rs_utils::{
    http_server::{add_max_cache_header, ServerError},
    image::scale_image_async,
};

use crate::Arhiv;

#[derive(Deserialize, Debug)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

#[tracing::instrument(skip(arhiv), level = "debug")]
pub async fn image_handler(
    arhiv: State<Arc<Arhiv>>,
    Path(blob_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let blob_id = BLOBId::from_string(blob_id)?;

    let baza = arhiv.baza.open()?;
    let blob_reader = baza.get_blob(&blob_id)?;
    let buf_reader = BufReader::new(blob_reader);
    let asset = get_asset_by_blob_id(&baza, &blob_id).context("Failed to find asset by blob id")?;

    let original_size = asset.data.size;

    let body = scale_image_async(buf_reader, params.max_w, params.max_h)
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
