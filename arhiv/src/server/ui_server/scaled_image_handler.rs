use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::{self, HeaderMapExt};

use baza::entities::Id;
use rs_utils::{http_server::ServerError, log::tracing};

use super::{scaled_images_cache::ImageParams, ServerContext};

#[tracing::instrument(skip(ctx), level = "debug")]
pub async fn scaled_image_handler(
    ctx: State<ServerContext>,
    Path(asset_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<impl IntoResponse, ServerError> {
    let asset_id: Id = asset_id.into();

    if params.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Image params must not be empty").into_response());
    }

    {
        let baza = ctx.arhiv.baza.open()?;

        let asset = if let Some(asset) = baza.get_asset(&asset_id)? {
            asset
        } else {
            return Ok(StatusCode::NOT_FOUND.into_response());
        };

        if !asset.data.is_image() {
            return Ok((StatusCode::BAD_REQUEST, "Asset is not an image").into_response());
        }
    }

    let data = ctx
        .img_cache
        .get_image(&asset_id, params, &ctx.arhiv.baza)
        .await?;

    let mut headers = HeaderMap::new();
    headers.typed_insert(headers::ContentType::from_str("image/webp")?);
    headers.typed_insert(headers::ContentLength(data.len() as u64));

    Ok((StatusCode::OK, headers, data).into_response())
}
