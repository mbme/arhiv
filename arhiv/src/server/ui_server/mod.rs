use std::{
    panic::AssertUnwindSafe,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::{
    extract::{CookieJar, cookie::Cookie},
    response::JavaScript,
};
use serde::Deserialize;
use serde_json::Value;

use crate::server::{
    AuthToken,
    http_server::{ServerError, add_no_cache_headers, fallback_route},
};
use baza::{BazaManager, DEV_MODE, entities::Id};
use baza_common::{get_crate_version, log};

use crate::{
    Arhiv,
    ui::dto::{APIRequest, ArhivUIConfig},
};

use self::api_handler::handle_api_request;
use self::assets_handler::{assets_handler, create_asset_handler};
use self::public_assets_handler::public_assets_handler;
use self::scaled_image_handler::scaled_image_handler;
use self::scaled_images_cache::ScaledImagesCache;

mod api_handler;
mod assets_handler;
mod public_assets_handler;
mod scaled_image_handler;
mod scaled_images_cache;

pub const UI_BASE_PATH: &str = "/ui";

pub const BROWSER_BOOTSTRAP_PATH: &str = "/auth";

pub const HEALTH_PATH: &str = "/health";

#[derive(Clone)]
pub struct ServerContext {
    pub arhiv: Arc<Arhiv>,
    pub img_cache: Arc<ScaledImagesCache>,
}

#[derive(Clone)]
struct BrowserBootstrap {
    auth_token: AuthToken,
    token: Arc<Mutex<Option<AuthToken>>>,
}

pub fn build_ui_router(
    auth_token: AuthToken,
    browser_bootstrap_token: AuthToken,
    arhiv: Arc<Arhiv>,
) -> Router<()> {
    let img_cache_dir = format!("{}/img-cache", arhiv.baza.get_state_dir());
    let img_cache = ScaledImagesCache::new(img_cache_dir);

    let ctx = ServerContext {
        arhiv,
        img_cache: Arc::new(img_cache),
    };

    let ui_auth_token = auth_token.clone();
    let ui_router = Router::new()
        .route("/", get(index_page))
        .route("/config.js", get(config_handler))
        .route("/api", post(api_handler))
        .route("/assets", post(create_asset_handler))
        .route("/assets/{asset_id}", get(assets_handler))
        .route("/assets/images/{asset_id}", get(scaled_image_handler))
        .layer(middleware::from_fn(no_cache_middleware))
        .route("/{*fileName}", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn_with_state(
            Arc::new(ui_auth_token),
            client_authenticator,
        ))
        .layer(middleware::from_fn(catch_panic_middleware))
        .with_state(ctx);

    let browser_bootstrap = BrowserBootstrap {
        auth_token: auth_token.clone(),
        token: Arc::new(Mutex::new(Some(browser_bootstrap_token))),
    };

    Router::new()
        .nest(UI_BASE_PATH, ui_router)
        .route(BROWSER_BOOTSTRAP_PATH, get(browser_bootstrap_handler))
        .route(HEALTH_PATH, get(health_handler))
        .fallback(fallback_route)
        .with_state(browser_bootstrap)
}

#[tracing::instrument(skip(ctx), level = "debug")]
async fn config_handler(ctx: State<ServerContext>) -> Result<impl IntoResponse, ServerError> {
    let arhiv = &ctx.arhiv;

    let config = serde_json::to_string_pretty(&ArhivUIConfig {
        storage_dir: arhiv.baza.get_storage_dir(),
        base_path: UI_BASE_PATH,
        schema: arhiv.baza.get_schema(),
        use_local_storage: true,
        min_password_length: BazaManager::MIN_PASSWORD_LENGTH,
        arhiv_missing: !arhiv.baza.storage_exists()?,
        arhiv_key_missing: !arhiv.baza.key_exists()?,
        arhiv_locked: arhiv.baza.is_locked(),
        dev_mode: DEV_MODE,
        arhiv_version: get_crate_version(),
        document_id_length: Id::LENGTH,
    })
    .context("Failed to serialize ArhivUI config")?;

    let content = format!("window.CONFIG = {config}");

    Ok(JavaScript(content))
}

#[tracing::instrument(level = "debug")]
async fn index_page() -> Result<impl IntoResponse, ServerError> {
    let content = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en" dir="ltr">
                <head>
                    <title>{}</title>

                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <meta http-equiv="Content-Security-Policy" content="script-src 'self'; ">

                    <link rel="icon" type="image/svg+xml" href="{UI_BASE_PATH}/favicon.svg" />
                    <link rel="stylesheet" href="{UI_BASE_PATH}/index.css" />
                </head>
                <body>
                    <main></main>

                    <script src="{UI_BASE_PATH}/config.js"></script>
                    <script src="{UI_BASE_PATH}/index.js"></script>
                </body>
            </html>"#,
        if DEV_MODE { "Arhiv DEV" } else { "Arhiv" }
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

#[derive(Deserialize)]
struct BrowserBootstrapQuery {
    token: Option<String>,
}

async fn client_authenticator(
    jar: CookieJar,
    State(server_auth_token): State<Arc<AuthToken>>,
    request: Request,
    next: Next,
) -> Response {
    let auth_token: Option<AuthToken> = if let Some(auth_token) = jar.get("AuthToken") {
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

    let mut response = next.run(request).await;

    set_auth_token_cookie(response.headers_mut(), &auth_token);

    response
}

async fn browser_bootstrap_handler(
    State(browser_bootstrap): State<BrowserBootstrap>,
    Query(BrowserBootstrapQuery { token }): Query<BrowserBootstrapQuery>,
) -> Response {
    let Some(token) = token else {
        return (
            StatusCode::UNAUTHORIZED,
            "Browser bootstrap token is missing",
        )
            .into_response();
    };

    let token = match AuthToken::parse(&token) {
        Ok(token) => token,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Invalid browser bootstrap token").into_response();
        }
    };

    let mut expected_token = match browser_bootstrap.token.lock() {
        Ok(token) => token,
        Err(_) => {
            log::error!("Browser bootstrap token lock is poisoned");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if expected_token.as_ref() != Some(&token) {
        return (StatusCode::UNAUTHORIZED, "Invalid browser bootstrap token").into_response();
    }

    *expected_token = None;

    let mut response = Redirect::temporary(UI_BASE_PATH).into_response();
    set_auth_token_cookie(response.headers_mut(), &browser_bootstrap.auth_token);

    response
}

fn set_auth_token_cookie(headers: &mut HeaderMap, auth_token: &AuthToken) {
    let auth_token_cookie = Cookie::build(("AuthToken", auth_token.serialize()))
        .path(UI_BASE_PATH)
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .build()
        .to_string();

    headers.append(
        axum::http::header::SET_COOKIE,
        auth_token_cookie
            .parse()
            .expect("Failed to convert AuthToken cookie into header value"),
    );
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

async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_bootstrap_token_is_single_use() {
        let auth_token = AuthToken::generate();
        let bootstrap_token = AuthToken::generate();
        let browser_bootstrap = BrowserBootstrap {
            auth_token,
            token: Arc::new(Mutex::new(Some(bootstrap_token.clone()))),
        };
        let query = BrowserBootstrapQuery {
            token: Some(bootstrap_token.serialize()),
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let first_response = runtime.block_on(browser_bootstrap_handler(
            State(browser_bootstrap.clone()),
            Query(query),
        ));
        let second_response = runtime.block_on(browser_bootstrap_handler(
            State(browser_bootstrap),
            Query(BrowserBootstrapQuery {
                token: Some(bootstrap_token.serialize()),
            }),
        ));

        assert_eq!(first_response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(second_response.status(), StatusCode::UNAUTHORIZED);
    }
}
