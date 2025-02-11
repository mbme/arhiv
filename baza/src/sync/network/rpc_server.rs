use std::{ops::Bound, str::FromStr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{DefaultBodyLimit, Path},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::{
    headers::{self, HeaderMapExt},
    TypedHeader,
};

use rs_utils::{
    create_body_from_file,
    http_server::{add_max_cache_header, ServerCertificate, ServerError},
    log, now,
};

use crate::{entities::BLOBId, sync::changeset::ChangesetRequest, Baza};

use super::auth::client_authenticator;

/// WARN: This router requires Extension<Arc<Baza>> to be available
pub fn build_rpc_router(server_certificate_der: Vec<u8>) -> Result<Router> {
    let router = Router::new()
        .route("/changeset", post(fetch_changes_handler))
        .route("/blobs/:blob_id", get(get_blob_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(client_authenticator))
        .layer(Extension(ServerCertificate::new(server_certificate_der)));

    Ok(router)
}

#[tracing::instrument(skip(baza), level = "debug")]
async fn get_blob_handler(
    Extension(baza): Extension<Arc<Baza>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&baza, &blob_id, &range.map(|range| range.0)).await
}

#[tracing::instrument(skip(baza), level = "debug")]
async fn fetch_changes_handler(
    Extension(baza): Extension<Arc<Baza>>,
    Json(request): Json<ChangesetRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let tx = baza.get_tx()?;
    let rev = tx.get_db_rev()?;
    let data_version = tx.get_data_version()?;

    if request.data_version != data_version {
        return Ok((
            StatusCode::CONFLICT,
            format!(
                "Requested data version {} doesn't match the data version {} of this instance",
                request.data_version, data_version
            ),
        )
            .into_response());
    }

    let changeset = tx.get_changeset(&request.rev)?;
    tx.set_last_sync_time(&now())?;
    tx.commit()?;

    if request.rev.is_concurrent_or_newer_than(&rev) {
        log::info!("Instance is outdated, comparing to {}", request.instance_id);
    }

    Ok(Json(changeset).into_response())
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
        .map(|range| range.satisfiable_ranges(size).collect::<Vec<_>>())
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
