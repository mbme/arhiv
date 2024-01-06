use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive},
        Html, IntoResponse, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use serde::Serialize;
use serde_json::Value;
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};

use baza::{entities::BLOBId, sync::respond_with_blob};
use rs_utils::{
    http_server::{add_no_cache_headers, ServerError},
    log,
};

use crate::dto::APIRequest;
use crate::Arhiv;

use self::api_handler::handle_api_request;
use self::image_handler::image_handler;
use self::public_assets_handler::public_assets_handler;

mod api_handler;
mod image_handler;
mod public_assets_handler;

pub const UI_BASE_PATH: &str = "/ui";

pub fn build_ui_router() -> Router<Arc<Arhiv>> {
    Router::new()
        .route("/", get(index_page))
        .route("/api", post(api_handler))
        .route("/events", get(events_handler))
        .route("/blobs/:blob_id", get(blob_handler))
        .route("/blobs/images/:blob_id", get(image_handler))
        .route("/*fileName", get(public_assets_handler))
}

#[derive(Serialize)]
struct Features {
    scraper: bool,
    use_local_storage: bool,
}

async fn index_page(State(arhiv): State<Arc<Arhiv>>) -> Result<impl IntoResponse, ServerError> {
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

#[tracing::instrument(skip(arhiv, request_value), level = "debug")]
async fn api_handler(
    State(arhiv): State<Arc<Arhiv>>,
    Json(request_value): Json<Value>,
) -> Result<impl IntoResponse, ServerError> {
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
    State(arhiv): State<Arc<Arhiv>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&arhiv.baza, &blob_id, &range.map(|val| val.0)).await
}

#[tracing::instrument(skip(arhiv), level = "debug")]
async fn events_handler(
    State(arhiv): State<Arc<Arhiv>>,
) -> Sse<impl Stream<Item = Result<Event, anyhow::Error>>> {
    let stream = BroadcastStream::new(arhiv.baza.get_events_channel()).map(|result| {
        result
            .map_err(|err| anyhow!("Event stream failed: {err}"))
            .and_then(|baza_event| {
                log::debug!("Sending BazaEvent {baza_event}");

                Event::default()
                    .json_data(baza_event)
                    .context("Event serialization failed")
            })
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
