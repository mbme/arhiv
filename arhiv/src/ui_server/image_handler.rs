use std::{fs, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    headers::{self, HeaderMapExt},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;

use baza::entities::BLOBId;
use rs_utils::{
    http_server::{add_max_cache_header, ServerError},
    image::scale_image,
    log,
};

use crate::Arhiv;

#[derive(Deserialize, Debug)]
pub struct ImageParams {
    pub max_w: Option<u32>,
    pub max_h: Option<u32>,
}

#[tracing::instrument(skip(arhiv))]
pub async fn image_handler(
    State(arhiv): State<Arc<Arhiv>>,
    Path(blob_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let blob_id = BLOBId::from_string(blob_id);

    let blob = arhiv
        .baza
        .get_connection()?
        .get_existing_blob(&blob_id)?
        .context("BLOB is missing")?;

    let (mut headers, body) =
        tokio::task::spawn_blocking(move || -> Result<(HeaderMap, Vec<u8>), anyhow::Error> {
            match scale_image(&blob.file_path, params.max_w, params.max_h) {
                Ok(body) => {
                    let mut headers = HeaderMap::new();
                    add_max_cache_header(&mut headers);
                    headers.typed_insert(headers::ContentType::png());

                    Ok((headers, body))
                }
                Err(err) => {
                    log::warn!("Failed to scale attachment image blob {blob_id}: {err}");

                    let mut headers = HeaderMap::new();
                    headers.typed_insert(headers::ContentType::from_str(&blob.get_media_type()?)?);

                    let body = fs::read(&blob.file_path).context("failed to read blob")?;

                    Ok((headers, body))
                }
            }
        })
        .await??;

    headers.typed_insert(headers::ContentLength(body.len() as u64));

    Ok((StatusCode::OK, headers, body))
}
