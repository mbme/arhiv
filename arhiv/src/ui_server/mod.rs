use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive},
        Html, IntoResponse, Sse,
    },
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::{future::Shared, FutureExt};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::oneshot;
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

use baza::{entities::BLOBId, sync::respond_with_blob};
use rs_utils::{
    http_server::{add_no_cache_headers, ServerError},
    log,
};

use crate::dto::APIRequest;

use self::api_handler::handle_api_request;
use self::image_handler::image_handler;
use self::public_assets_handler::public_assets_handler;
pub use self::state::UIState;

mod api_handler;
mod image_handler;
mod public_assets_handler;
mod state;

pub const UI_BASE_PATH: &str = "/ui";

pub fn build_ui_router(shutdown_receiver: oneshot::Receiver<()>) -> Router<Arc<UIState>> {
    let shutdown_receiver = shutdown_receiver.shared();

    Router::new()
        .route("/", get(index_page))
        .route("/api", post(api_handler))
        .route("/events", get(events_handler))
        .route("/blobs/:blob_id", get(blob_handler))
        .route("/blobs/images/:blob_id", get(image_handler))
        .route("/*fileName", get(public_assets_handler))
        .layer(Extension(shutdown_receiver))
        .layer(DefaultBodyLimit::disable())
}

#[derive(Serialize)]
struct Features {
    scraper: bool,
    use_local_storage: bool,
}

async fn index_page(state: State<Arc<UIState>>) -> Result<impl IntoResponse, ServerError> {
    let arhiv = state.must_get_arhiv()?;

    let schema =
        serde_json::to_string(arhiv.baza.get_schema()).context("failed to serialize schema")?;

    let features = Features {
        scraper: cfg!(feature = "scraper"),
        use_local_storage: cfg!(target_os = "android"),
    };
    let features = serde_json::to_string(&features).context("failed to serialize features")?;

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

#[tracing::instrument(skip(state, shutdown_receiver), level = "debug")]
async fn events_handler(
    state: State<Arc<UIState>>,
    Extension(shutdown_receiver): Extension<Shared<oneshot::Receiver<()>>>,
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

    let shutdown_stream = shutdown_receiver.clone().into_stream().map(|_| Ok(None));

    let stream = futures::stream::select(events_stream, shutdown_stream)
        .map_while(|value| value.transpose());

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
