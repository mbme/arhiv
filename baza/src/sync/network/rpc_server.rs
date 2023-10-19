use std::{ops::Bound, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State, TypedHeader},
    headers::{self, HeaderMapExt},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use rs_utils::{
    create_body_from_file,
    http_server::{add_max_cache_header, ServerError},
    log,
};

use crate::{
    entities::{BLOBId, Revision},
    sync::Ping,
    Baza, BazaEvent,
};

pub fn build_rpc_router() -> Router<Arc<Baza>> {
    Router::new()
        .route("/ping", post(exchange_pings_handler))
        .route("/blobs/:blob_id", get(get_blob_handler))
        .route("/changeset/:min_rev", get(get_changeset_handler))
}

#[tracing::instrument(skip(baza), level = "debug")]
async fn get_blob_handler(
    State(baza): State<Arc<Baza>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&baza, &blob_id, &range.map(|range| range.0)).await
}

#[tracing::instrument(skip(baza), level = "debug")]
async fn exchange_pings_handler(
    State(baza): State<Arc<Baza>>,
    Json(other_ping): Json<Ping>,
) -> Result<impl IntoResponse, ServerError> {
    let ping = baza.get_connection()?.get_ping()?;

    if other_ping.rev.is_concurrent_or_newer_than(&ping.rev) {
        log::info!("Instance is outdated, comparing to {}", ping.instance_id);
        baza.publish_event(BazaEvent::InstanceOutdated {})?;
    }

    Ok(Json(ping))
}

#[tracing::instrument(skip(baza), level = "debug")]
async fn get_changeset_handler(
    State(baza): State<Arc<Baza>>,
    Path(min_rev): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let min_rev = serde_json::from_str(&min_rev).context("failed to parse min_rev")?;
    let min_rev = Revision::from_value(min_rev)?;

    let changeset = baza.get_connection()?.get_changeset(&min_rev)?;

    Ok(Json(changeset))
}

pub async fn respond_with_blob(
    baza: &Baza,
    blob_id: &BLOBId,
    range: &Option<headers::Range>,
) -> Result<Response, ServerError> {
    let conn = baza.get_connection()?;
    let blob = conn.get_blob(blob_id);

    if !blob.exists()? {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let size = blob.get_size()?;

    let mut headers = HeaderMap::new();
    add_max_cache_header(&mut headers);
    headers.typed_insert(headers::ContentLength(size));
    headers.typed_insert(headers::AcceptRanges::bytes());
    headers.typed_insert(headers::ContentType::from_str(&blob.get_media_type()?)?);

    let ranges = range
        .as_ref()
        .map(|range| range.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if ranges.len() == 1 {
        let (start_pos, end_pos) = ranges[0];

        let start_pos = match start_pos {
            Bound::Included(start_pos) => start_pos,
            Bound::Excluded(start_pos) => start_pos + 1,
            Bound::Unbounded => 0,
        };

        let end_pos = match end_pos {
            Bound::Included(end_pos) => end_pos,
            Bound::Excluded(end_pos) => end_pos - 1,
            Bound::Unbounded => size - 1,
        };

        if start_pos >= size || end_pos >= size {
            log::warn!(
                "blob {}: range {}-{} out of {} not satisfiable",
                blob_id,
                start_pos,
                end_pos,
                size
            );

            return Ok(StatusCode::RANGE_NOT_SATISFIABLE.into_response());
        }

        let range_size = end_pos + 1 - start_pos;
        let body = create_body_from_file(&blob.file_path, start_pos, Some(range_size)).await?;

        let content_range = headers::ContentRange::bytes(start_pos..end_pos, size)?;

        headers.typed_insert(content_range);

        Ok((StatusCode::PARTIAL_CONTENT, headers, body).into_response())
    } else {
        let body = create_body_from_file(&blob.file_path, 0, None).await?;

        Ok((StatusCode::OK, headers, body).into_response())
    }
}
