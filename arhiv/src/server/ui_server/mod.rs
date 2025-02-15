use std::sync::Arc;

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
    headers, TypedHeader,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use baza::{entities::BLOBId, schema::create_attachment, sync::respond_with_blob, Baza};
use rs_utils::{
    crypto_key::CryptoKey,
    http_server::{add_no_cache_headers, ServerError},
    log, stream_to_file, AuthToken, SecretString,
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
        .route("/blobs/:blob_id", get(blob_handler))
        .route("/blobs/images/:blob_id", get(image_handler))
        .route("/*fileName", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(client_authenticator))
        .layer(Extension(Arc::new(ui_key)))
}

#[derive(Deserialize)]
struct CreateArhivRequest {
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

    state.create_arhiv(create_arhiv_request.password)?;

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
    let min_password_length = Baza::MIN_PASSWORD_LENGTH;

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

    let temp_file = arhiv.baza.get_path_manager().new_temp_file("arhiv-blob");
    let stream = request.into_body().into_data_stream();

    stream_to_file(temp_file.open_tokio_file(0).await?, stream).await?;

    let mut tx = arhiv.baza.get_tx()?;

    let attachment = create_attachment(&mut tx, &temp_file.path, true, Some(file_name))?;

    tx.commit()?;

    Ok(attachment.id.to_string())
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
    auth_token: String,
}

async fn client_authenticator(
    jar: CookieJar,
    auth_token_query: Option<Query<AuthTokenQuery>>,
    Extension(ui_key): Extension<Arc<CryptoKey>>,
    request: Request,
    next: Next,
) -> Response {
    let auth_token: Option<AuthToken> =
        if let Some(Query(AuthTokenQuery { auth_token })) = auth_token_query {
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
