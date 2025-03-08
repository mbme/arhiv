use std::{io::Seek, ops::Bound, str::FromStr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::{
    extract::{cookie::Cookie, CookieJar},
    headers::{self, HeaderMapExt},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::{
    baza2::BazaManager,
    entities::Id,
    schema::{create_asset, Asset},
};
use rs_utils::{
    create_body_from_reader,
    crypto_key::CryptoKey,
    http_server::{add_no_cache_headers, ServerError},
    log, stream_to_file, AuthToken, TempFile,
};

use crate::{definitions::get_standard_schema, dto::APIRequest, Arhiv};

use self::api_handler::handle_api_request;
use self::image_handler::image_handler;
use self::public_assets_handler::public_assets_handler;

mod api_handler;
mod image_handler;
mod public_assets_handler;

pub const UI_BASE_PATH: &str = "/ui";

pub fn build_ui_router(ui_key: CryptoKey) -> Router<Arc<Arhiv>> {
    Router::new()
        .route("/", get(index_page))
        .route("/api", post(api_handler))
        .route("/assets", post(create_asset_handler))
        .route("/assets/{asset_id}", get(asset_handler))
        .route("/assets/images/{asset_id}", get(image_handler))
        .route("/{*fileName}", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(client_authenticator))
        .layer(middleware::from_fn(no_cache_middleware))
        .layer(Extension(Arc::new(ui_key)))
}

#[derive(Serialize)]
struct Features {
    use_local_storage: bool,
}

async fn index_page(arhiv: State<Arc<Arhiv>>) -> Result<impl IntoResponse, ServerError> {
    let create_arhiv = !arhiv.baza.storage_exists()?;
    let is_locked = arhiv.baza.is_locked();

    let schema =
        serde_json::to_string(&get_standard_schema()).context("failed to serialize schema")?;

    let features = Features {
        use_local_storage: true,
    };
    let features = serde_json::to_string(&features).context("failed to serialize features")?;
    let min_password_length = BazaManager::MIN_PASSWORD_LENGTH;

    let content = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en" dir="ltr">
                <head>
                    <title>Arhiv</title>

                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                    <link rel="icon" type="image/svg+xml" href="{UI_BASE_PATH}/favicon.svg" />
                    <link rel="stylesheet" href="{UI_BASE_PATH}/index.css" />
                </head>
                <body>
                    <main></main>

                    <script>
                        window.BASE_PATH = "{UI_BASE_PATH}";
                        window.SCHEMA = {schema};
                        window.FEATURES = {features};
                        window.MIN_PASSWORD_LENGTH = {min_password_length};
                        window.CREATE_ARHIV = {create_arhiv};
                        window.ARHIV_LOCKED = {is_locked};
                    </script>

                    <script src="{UI_BASE_PATH}/index.js"></script>
                </body>
            </html>"#
    );

    Ok(Html(content))
}

#[tracing::instrument(skip(arhiv, request_value), level = "debug")]
async fn api_handler(
    arhiv: State<Arc<Arhiv>>,
    Json(request_value): Json<Value>,
) -> Result<impl IntoResponse, ServerError> {
    log::info!(
        "API request: {}",
        request_value.get("typeName").unwrap_or(&Value::Null)
    );

    let request: APIRequest =
        serde_json::from_value(request_value).context("failed to parse APIRequest")?;
    let response = handle_api_request(&arhiv, request).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(arhiv, request), level = "debug")]
async fn create_asset_handler(
    arhiv: State<Arc<Arhiv>>,
    request: Request,
) -> Result<impl IntoResponse, ServerError> {
    let file_name = request
        .headers()
        .get("X-File-Name")
        .context("X-File-Name header is missing")?
        .to_str()
        .context("Failed to read X-File-Name header as a string")?
        .to_string();

    let temp_file = TempFile::new_in_downloads_dir("arhiv-asset")?;
    let stream = request.into_body().into_data_stream();

    stream_to_file(temp_file.open_tokio_file(0).await?, stream).await?;

    let mut baza = arhiv.baza.open_mut()?;

    let asset = create_asset(&mut baza, &temp_file.path, Some(file_name))?;

    baza.save_changes()?;

    Ok(asset.id.to_string())
}

async fn asset_handler(
    arhiv: State<Arc<Arhiv>>,
    Path(asset_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let asset_id: Id = asset_id.into();

    respond_with_blob(&arhiv.baza, &asset_id, &range.map(|val| val.0)).await
}

#[derive(Deserialize)]
struct AuthTokenQuery {
    #[serde(rename = "AuthToken")]
    auth_token: Option<String>,
}

/// Extract AuthToken either from url query param, or from the cookie
async fn client_authenticator(
    jar: CookieJar,
    auth_token_query: Query<AuthTokenQuery>,
    Extension(ui_key): Extension<Arc<CryptoKey>>,
    request: Request,
    next: Next,
) -> Response {
    let auth_token: Option<AuthToken> = if let Query(AuthTokenQuery {
        auth_token: Some(auth_token),
    }) = auth_token_query
    {
        match AuthToken::parse(&auth_token) {
            Ok(auth_token) => Some(auth_token),
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to parse AuthToken query param: {err}"),
                )
                    .into_response();
            }
        }
    } else if let Some(auth_token) = jar.get("AuthToken") {
        match AuthToken::parse(auth_token.value()) {
            Ok(auth_token) => Some(auth_token),
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to parse AuthToken cookie: {err}"),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let auth_token = if let Some(auth_token) = auth_token {
        auth_token
    } else {
        return (StatusCode::UNAUTHORIZED, "AuthToken is missing").into_response();
    };

    if let Err(err) = auth_token.assert_is_valid(&ui_key) {
        log::warn!("Got unauthenticated client: {err}");

        return (StatusCode::UNAUTHORIZED, "Invalid AuthToken").into_response();
    }

    let auth_token_cookie = Cookie::build(("AuthToken", auth_token.serialize()))
        .path("/")
        .http_only(true)
        .secure(true)
        .build()
        .to_string();

    let mut response = next.run(request).await;

    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        auth_token_cookie
            .parse()
            .expect("Failed to convert AuthToken cookie into header value"),
    );

    response
}

async fn respond_with_blob(
    baza_manager: &BazaManager,
    asset_id: &Id,
    range: &Option<headers::Range>,
) -> Result<Response, ServerError> {
    let (asset, mut blob) = {
        let baza = baza_manager.open()?;

        let asset: Asset = if let Some(head) = baza.get_document(asset_id) {
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

    let size = asset.data.size;

    let mut headers = HeaderMap::new();
    headers.typed_insert(headers::ContentLength(size));
    headers.typed_insert(headers::AcceptRanges::bytes());
    headers.typed_insert(headers::ContentType::from_str(&asset.data.media_type)?);

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

async fn no_cache_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    add_no_cache_headers(response.headers_mut());

    response
}
