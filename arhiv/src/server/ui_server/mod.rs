use std::{panic::AssertUnwindSafe, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{DefaultBodyLimit, Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::OnceCell;

use baza::baza2::BazaManager;
use rs_utils::{
    http_server::{add_no_cache_headers, fallback_route, ServerError},
    log::{self, tracing},
    stream_to_file, AuthToken, TempFile,
};

use crate::{
    dto::{APIRequest, ArhivUIConfig},
    Arhiv,
};

use self::api_handler::handle_api_request;
use self::assets_handler::assets_handler;
use self::public_assets_handler::public_assets_handler;
use self::scaled_image_handler::scaled_image_handler;
use self::scaled_images_cache::ScaledImagesCache;

mod api_handler;
mod assets_handler;
mod public_assets_handler;
mod scaled_image_handler;
mod scaled_images_cache;

pub const UI_BASE_PATH: &str = "/ui";

pub const HEALTH_PATH: &str = "/health";

#[derive(Clone)]
pub struct ServerContext {
    pub arhiv: Arc<Arhiv>,
    pub img_cache: Arc<ScaledImagesCache>,
    init: Arc<OnceCell<()>>,
}

pub fn build_ui_router(auth_token: AuthToken, arhiv: Arc<Arhiv>) -> Router<()> {
    let img_cache_dir = format!("{}/img-cache", arhiv.baza.get_state_dir());
    let img_cache = ScaledImagesCache::new(img_cache_dir, arhiv.baza.clone());

    let ctx = ServerContext {
        arhiv,
        img_cache: Arc::new(img_cache),
        init: Arc::new(OnceCell::const_new()),
    };

    let ui_router = Router::new()
        .route("/", get(index_page))
        .route("/api", post(api_handler))
        .route("/assets", post(create_asset_handler))
        .route("/assets/{asset_id}", get(assets_handler))
        .route("/assets/images/{asset_id}", get(scaled_image_handler))
        .layer(middleware::from_fn(no_cache_middleware))
        .route("/{*fileName}", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            init_server_context_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::new(auth_token),
            client_authenticator,
        ))
        .layer(middleware::from_fn(catch_panic_middleware))
        .with_state(ctx);

    Router::new()
        .nest(UI_BASE_PATH, ui_router)
        .route(HEALTH_PATH, get(health_handler))
        .fallback(fallback_route)
}

#[tracing::instrument(skip(ctx), level = "debug")]
async fn index_page(ctx: State<ServerContext>) -> Result<impl IntoResponse, ServerError> {
    let arhiv = &ctx.arhiv;

    let config = serde_json::to_string_pretty(&ArhivUIConfig {
        storage_dir: arhiv.baza.get_storage_dir(),
        base_path: UI_BASE_PATH,
        schema: arhiv.baza.get_schema(),
        use_local_storage: true,
        min_password_length: BazaManager::MIN_PASSWORD_LENGTH,
        create_arhiv: !arhiv.baza.storage_exists()?,
        arhiv_locked: arhiv.baza.is_locked(),
    })
    .context("Failed to serialize ArhivUI config")?;

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
                        window.CONFIG = {config};
                    </script>

                    <script src="{UI_BASE_PATH}/index.js"></script>
                </body>
            </html>"#
    );

    Ok(Html(content))
}

#[tracing::instrument(skip(ctx, request_value), level = "debug")]
async fn api_handler(
    ctx: State<ServerContext>,
    Json(request_value): Json<Value>,
) -> Result<impl IntoResponse, ServerError> {
    log::info!(
        "API request: {}",
        request_value
            .get("typeName")
            .unwrap_or(&serde_json::Value::Null)
    );

    let request: APIRequest =
        serde_json::from_value(request_value).context("failed to parse APIRequest")?;
    let response = handle_api_request(&ctx, request).await?;

    Ok(Json(response))
}

#[tracing::instrument(skip(ctx, request), level = "debug")]
async fn create_asset_handler(
    ctx: State<ServerContext>,
    request: Request,
) -> Result<impl IntoResponse, ServerError> {
    let file_name = request
        .headers()
        .get("X-File-Name")
        .context("X-File-Name header is missing")?
        .to_str()
        .context("Failed to read X-File-Name header as a string")?
        .to_string();

    let arhiv = &ctx.arhiv;

    let temp_file = TempFile::new_in_dir(arhiv.baza.get_downloads_dir(), "arhiv-asset");
    let stream = request.into_body().into_data_stream();

    stream_to_file(temp_file.open_tokio_file(0).await?, stream).await?;

    let mut baza = arhiv.baza.open_mut()?;

    let mut asset = baza.create_asset(&temp_file.path)?;
    asset.data.filename = file_name;

    let document = asset.into_document()?;
    let document = baza
        .stage_document(document, &None)
        .context("Failed to update asset filename")?;

    let asset_id = document.id.to_string();

    baza.save_changes()?;

    Ok(asset_id)
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
    State(server_auth_token): State<Arc<AuthToken>>,
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

    if auth_token != *server_auth_token {
        log::warn!("Got client with an invalid auth token");

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

async fn no_cache_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    add_no_cache_headers(response.headers_mut());

    response
}

async fn catch_panic_middleware(req: Request, next: Next) -> Response {
    use futures::FutureExt;

    let result = AssertUnwindSafe(next.run(req)).catch_unwind().await;

    match result {
        Ok(response) => response,
        Err(err) => {
            let err = if let Some(s) = err.downcast_ref::<String>() {
                s.as_str()
            } else if let Some(s) = err.downcast_ref::<&str>() {
                s
            } else {
                ""
            };

            log::error!("Panic: {err}");

            (StatusCode::INTERNAL_SERVER_ERROR, format!("Panic: {err}")).into_response()
        }
    }
}

async fn init_server_context_middleware(
    ctx: State<ServerContext>,
    req: Request,
    next: Next,
) -> Response {
    let arhiv = &ctx.arhiv;

    // init server context on first request
    ctx.init
        .get_or_try_init(|| async {
            log::info!("Initializing UI server context");

            if arhiv.baza.storage_exists()? {
                match arhiv.unlock_using_keyring() {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to use keyring: {err}");
                    }
                }
            }

            if arhiv.baza.is_unlocked() {
                ctx.img_cache.init().await?;
            }

            Ok::<(), anyhow::Error>(())
        })
        .await
        .expect("Failed to initialize UI server context");

    next.run(req).await
}

async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}
