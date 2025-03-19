use std::{io::Seek, ops::Bound, str::FromStr};

use axum::{
    extract::{Path, State},
    http::{self, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{self, HeaderMapExt},
    TypedHeader,
};

use baza::entities::Id;
use rs_utils::{
    create_body_from_reader,
    http_server::ServerError,
    log::{self, tracing},
};

use super::ServerContext;

#[tracing::instrument(skip(ctx), level = "debug")]
pub async fn assets_handler(
    ctx: State<ServerContext>,
    Path(asset_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> Result<Response, ServerError> {
    let asset_id: Id = asset_id.into();

    let (asset, mut blob) = {
        let baza = ctx.arhiv.baza.open()?;

        let asset = baza.get_asset(&asset_id)?;
        let asset = if let Some(asset) = asset {
            asset
        } else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        let blob = baza.get_asset_data(&asset_id)?;

        (asset, blob)
    };

    let size = asset.data.size;

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            r#"attachment; filename="{}""#,
            asset.data.filename
        ))?,
    );
    headers.typed_insert(headers::ContentLength(size));
    headers.typed_insert(headers::AcceptRanges::bytes());
    headers.typed_insert(headers::ContentType::from_str(&asset.data.media_type)?);

    let ranges = range
        .as_ref()
        .map(|range| range.0.satisfiable_ranges(size).collect::<Vec<_>>())
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
                "Asset {asset_id}: range {start_pos}-{end_pos} out of {size} not satisfiable"
            );

            return Ok(StatusCode::RANGE_NOT_SATISFIABLE.into_response());
        }

        blob.seek(std::io::SeekFrom::Start(start_pos))?;

        let range_size = end_pos + 1 - start_pos;
        let body = create_body_from_reader(blob, Some(range_size)).await?;

        let content_range = headers::ContentRange::bytes(start_pos..end_pos, size)?;

        headers.typed_insert(content_range);

        Ok((StatusCode::PARTIAL_CONTENT, headers, body).into_response())
    } else {
        let body = create_body_from_reader(blob, None).await?;

        Ok((StatusCode::OK, headers, body).into_response())
    }
}
