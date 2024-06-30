use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive},
        Html, IntoResponse, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

use baza::{entities::BLOBId, sync::respond_with_blob, Credentials};
use rs_utils::{
    http_server::{add_no_cache_headers, ServerError},
    log, SecretString,
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

pub fn build_ui_router() -> Router<Arc<UIState>> {
    Router::new()
        .route("/", get(index_page))
        .route("/create", post(create_arhiv_handler))
        .route("/api", post(api_handler))
        .route("/events", get(events_handler))
        .route("/blobs/:blob_id", get(blob_handler))
        .route("/blobs/images/:blob_id", get(image_handler))
        .route("/*fileName", get(public_assets_handler))
        .layer(DefaultBodyLimit::disable())
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
    scraper: bool,
    use_local_storage: bool,
}

async fn index_page(state: State<Arc<UIState>>) -> Result<impl IntoResponse, ServerError> {
    let create_arhiv = !state.arhiv_exists()?;

    let schema =
        serde_json::to_string(&get_standard_schema()).context("failed to serialize schema")?;

    let features = Features {
        scraper: cfg!(feature = "scraper"),
        use_local_storage: true,
    };
    let features = serde_json::to_string(&features).context("failed to serialize features")?;
    let min_login_length = Credentials::MIN_LOGIN_LENGTH;
    let min_password_length = Credentials::MIN_PASSWORD_LENGTH;

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

async fn blob_handler(
    state: State<Arc<UIState>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let arhiv = state.must_get_arhiv()?;

    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&arhiv.baza, &blob_id, &range.map(|val| val.0)).await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn events_handler(
    state: State<Arc<UIState>>,
) -> Result<Sse<impl Stream<Item = anyhow::Result<Event>>>, ServerError> {
    let arhiv = state.must_get_arhiv()?;

    let events_stream = BroadcastStream::new(arhiv.baza.get_events_channel()).map(|result| {
        let baza_event = result.context("Event stream failed")?;

        log::debug!("Sending BazaEvent {baza_event}");

        let event = Event::default()
            .json_data(baza_event)
            .context("Event serialization failed")?;

        Ok(Some(event))
    });

    let shutdown_stream = state
        .shutdown_receiver
        .clone()
        .into_stream()
        .map(|_| Ok(None))
        .take(1);

    let stream = futures::stream::select(events_stream, shutdown_stream)
        .map_while(|value| value.transpose());

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
