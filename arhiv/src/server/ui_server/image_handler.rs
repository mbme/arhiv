use std::{
    fmt::Display,
    io::{BufReader, Cursor},
    str::FromStr,
    sync::Arc,
};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::{self, HeaderMapExt};
use serde::Deserialize;

use baza::{entities::Id, schema::Asset};
use rs_utils::{
    get_string_hash_sha256, http_server::ServerError, image::scale_image_async, log, read_all,
};

use crate::Arhiv;

#[derive(Deserialize, Debug)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

impl ImageParams {
    pub fn is_empty(&self) -> bool {
        self.max_w.is_none() && self.max_h.is_none()
    }
}

impl Display for ImageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.max_w, self.max_h,)
    }
}

#[tracing::instrument(skip(arhiv), level = "debug")]
pub async fn image_handler(
    arhiv: State<Arc<Arhiv>>,
    Path(asset_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let asset_id: Id = asset_id.into();

    if params.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Image params must not be empty").into_response());
    }

    let mut headers = HeaderMap::new();
    headers.typed_insert(headers::ContentType::from_str("image/webp")?);

    let cache_file_name = get_string_hash_sha256(&format!("{asset_id}-{params}-webp"));

    let (asset, blob) = {
        let baza = arhiv.baza.open()?;

        if baza.cache_file_exists(&cache_file_name)? {
            log::debug!("Found cached image file for {asset_id} {params}");
            let cache_file_reader = baza.get_cache_file(&cache_file_name)?;

            let data = read_all(cache_file_reader)?;

            headers.typed_insert(headers::ContentLength(data.len() as u64));

            return Ok((StatusCode::OK, headers, data).into_response());
        }

        let asset: Asset = if let Some(head) = baza.get_document(&asset_id) {
            head.get_single_document()
                .clone()
                .convert()
                .context("Document is not an asset")?
        } else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        if !asset.data.is_image() {
            return Ok((StatusCode::BAD_REQUEST, "Asset is not an image").into_response());
        }

        let blob = baza.get_blob(&asset.data.blob)?;

        (asset, blob)
    };

    log::info!("Scaling image {asset_id} to {params}");

    let buf_reader = BufReader::new(blob);
    let body = scale_image_async(buf_reader, params.max_w, params.max_h)
        .await
        .context("failed to scale image")?;

    let scaled_img_size = body.len() as u64;
    let original_size = asset.data.size;

    let scaled_img_size_percent = scaled_img_size * 100 / original_size;

    log::info!(
        "Scaled image from {original_size} bytes to {scaled_img_size} bytes: to {scaled_img_size_percent}%",
    );

    let mut baza = arhiv.baza.open_mut()?;

    log::debug!("Saving scaled image {asset_id} to cache");
    let mut body = Cursor::new(body);
    baza.add_cache_file(&cache_file_name, &mut body)?;
    let body = body.into_inner();

    headers.typed_insert(headers::ContentLength(scaled_img_size));

    Ok((StatusCode::OK, headers, body).into_response())
}
