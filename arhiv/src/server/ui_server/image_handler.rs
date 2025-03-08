use std::{io::BufReader, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::{self, HeaderMapExt};
use serde::Deserialize;

use baza::{entities::Id, schema::Asset};
use rs_utils::{http_server::ServerError, image::scale_image_async, log};

use crate::Arhiv;

#[derive(Deserialize, Debug)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

#[tracing::instrument(skip(arhiv), level = "debug")]
pub async fn image_handler(
    arhiv: State<Arc<Arhiv>>,
    Path(asset_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let asset_id: Id = asset_id.into();

    let (asset, blob) = {
        let baza = arhiv.baza.open()?;

        let asset: Asset = if let Some(head) = baza.get_document(&asset_id) {
            head.get_single_document()
                .clone()
                .convert()
                .context("Document is not an asset")?
        } else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        let blob = baza.get_blob(&asset.data.blob)?;

        (asset, blob)
    };

    let original_size = asset.data.size;

    let buf_reader = BufReader::new(blob);
    let body = scale_image_async(buf_reader, params.max_w, params.max_h)
        .await
        .context("failed to scale image")?;

    let scaled_img_size = body.len();

    let scaled_img_size_percent = scaled_img_size as u64 * 100 / original_size;

    log::info!(
        "scaled image from {original_size} bytes to {scaled_img_size} bytes: to {scaled_img_size_percent}%",
    );

    let mut headers = HeaderMap::new();
    headers.typed_insert(headers::ContentType::from_str("image/webp")?);
    headers.typed_insert(headers::ContentLength(body.len() as u64));

    Ok((StatusCode::OK, headers, body).into_response())
}
