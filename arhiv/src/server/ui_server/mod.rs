use std::{ops::Bound, str::FromStr, sync::Arc};

use anyhow::{anyhow, Context};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::HeaderMap,
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
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::{
    baza2::{Baza, BazaManager},
    entities::BLOBId,
    schema::create_asset,
};
use rs_utils::{
    create_body_from_file,
    crypto_key::CryptoKey,
    http_server::{add_max_cache_header, add_no_cache_headers, ServerError},
    log, stream_to_file, AuthToken, SecretString, TempFile,
};

use crate::{definitions::get_standard_schema, dto::APIRequest};

use self::api_handler::handle_api_request;
use self::image_handler::image_handler;
use self::public_assets_handler::public_assets_handler;
pub use self::state::UIState;

mod api_handler;
mod image_handler;
mod public_assets_handler;
mod state;

pub const UI_BASE_PATH: &str = "/ui";

pub fn build_ui_router(ui_key: CryptoKey) -> Router<Arc<UIState>> {
    Router::new()
        .route("/", get(index_page))
        .route("/create", post(create_arhiv_handler))
        .route("/api", post(api_handler))
        .route("/blobs", post(create_blob_handler))
        .route("/blobs/{blob_id}", get(blob_handler))
        .route("/blobs/images/{blob_id}", get(image_handler))
        .route("/{*fileName}", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(client_authenticator))
        .layer(Extension(Arc::new(ui_key)))
}

#[derive(Deserialize)]
struct CreateArhivRequest {
    login: String,
    password: SecretString,
}

async fn create_arhiv_handler(
    state: State<Arc<UIState>>,
    Json(create_arhiv_request): Json<CreateArhivRequest>,
) -> Result<impl IntoResponse, ServerError> {
    if state.arhiv_exists()? {
        return Err(anyhow!("Arhiv already exists").into());
    }

    log::info!("Creating new arhiv");

    let auth = Credentials::new(create_arhiv_request.login, create_arhiv_request.password)?;

    state.create_arhiv(auth)?;

    Ok(())
}

#[derive(Serialize)]
struct Features {
    use_local_storage: bool,
}

async fn index_page(state: State<Arc<UIState>>) -> Result<impl IntoResponse, ServerError> {
    let create_arhiv = !state.arhiv_exists()?;

    let schema =
        serde_json::to_string(&get_standard_schema()).context("failed to serialize schema")?;

    let features = Features {
        use_local_storage: true,
    };
    let features = serde_json::to_string(&features).context("failed to serialize features")?;
    let min_login_length = BazaManager::MIN_LOGIN_LENGTH;
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
                        window.MIN_LOGIN_LENGTH = {min_login_length};
                        window.MIN_PASSWORD_LENGTH = {min_password_length};
                        window.CREATE_ARHIV = {create_arhiv};
                    </script>

                    <script src="{UI_BASE_PATH}/index.js"></script>
                </body>
            </html>"#
    );

    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    Ok((headers, Html(content)))
}

#[tracing::instrument(skip(state, request_value), level = "debug")]
async fn api_handler(
    state: State<Arc<UIState>>,
    Json(request_value): Json<Value>,
) -> Result<impl IntoResponse, ServerError> {
    let arhiv = state.must_get_arhiv()?;

    log::info!(
        "API request: {}",
        request_value.get("typeName").unwrap_or(&Value::Null)
    );

    let request: APIRequest =
        serde_json::from_value(request_value).context("failed to parse APIRequest")?;
    let response = handle_api_request(&arhiv, request).await?;

    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    Ok((headers, Json(response)))
}

#[tracing::instrument(skip(state, request), level = "debug")]
async fn create_blob_handler(
    state: State<Arc<UIState>>,
    request: Request,
) -> Result<impl IntoResponse, ServerError> {
    let arhiv = state.must_get_arhiv()?;

    let file_name = request
        .headers()
        .get("X-File-Name")
        .context("X-File-Name header is missing")?
        .to_str()
        .context("Failed to read X-File-Name header as a string")?
        .to_string();

    let temp_file = TempFile::new_in_downloads_dir("arhiv-blob")?;
    let stream = request.into_body().into_data_stream();

    stream_to_file(temp_file.open_tokio_file(0).await?, stream).await?;

    let mut baza = arhiv.baza.open()?;

    let asset = create_asset(&mut baza, &temp_file.path, Some(file_name))?;

    baza.save_changes()?;

    Ok(asset.id.to_string())
}

async fn blob_handler(
    state: State<Arc<UIState>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let arhiv = state.must_get_arhiv()?;

    let blob_id = BLOBId::from_string(blob_id)?;

    respond_with_blob(&arhiv.baza, &blob_id, &range.map(|val| val.0)).await
}

#[derive(Deserialize)]
struct AuthTokenQuery {
    #[serde(rename = "AuthToken")]
    auth_token: Option<String>,
}

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
    baza: &Baza,
    blob_id: &BLOBId,
    range: &Option<headers::Range>,
) -> Result<Response, ServerError> {
    let blob = baza.get_blob(blob_id);

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
