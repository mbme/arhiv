use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    headers,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router, TypedHeader,
};

use baza::{entities::BLOBId, sync::respond_with_blob};
use rs_utils::http_server::{no_cache_headers, ServerError};

use crate::dto::APIRequest;
use crate::Arhiv;

use self::api_handler::handle_api_request;
use self::public_assets_handler::public_assets_handler;

mod api_handler;
mod public_assets_handler;

pub fn build_ui_router() -> Router<Arc<Arhiv>> {
    // TODO logger_middleware
    // TODO not_found_handler
    // TODO error_handler
    Router::new()
        .route("/", get(index_page))
        .route("/blobs/:blob_id", get(blob_handler))
        .route("/*fileName", get(public_assets_handler))
        .route("/api", post(api_handler))
}

async fn index_page(State(arhiv): State<Arc<Arhiv>>) -> Result<impl IntoResponse, ServerError> {
    let schema =
        serde_json::to_string(arhiv.baza.get_schema()).context("failed to serialize schema")?;

    let base_path = "/ui";

    let content = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en" dir="ltr">
                <head>
                    <title>Arhiv</title>

                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                    <link rel="icon" type="image/svg+xml" href="{base_path}/favicon.svg" />
                    <link rel="stylesheet" href="{base_path}/index.css" />
                </head>
                <body>
                    <main></main>

                    <script>
                        window.BASE_PATH = "{base_path}";
                        window.SCHEMA = {schema};
                    </script>

                    <script src="{base_path}/index.js"></script>
                </body>
            </html>"#
    );

    Ok((no_cache_headers(), Html(content)))
}

async fn api_handler(
    State(arhiv): State<Arc<Arhiv>>,
    Json(request): Json<APIRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let response = handle_api_request(&arhiv, request).await?;

    Ok((no_cache_headers(), Json(response)))
}

async fn blob_handler(
    State(arhiv): State<Arc<Arhiv>>,
    Path(blob_id): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
) -> impl IntoResponse {
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&arhiv.baza, &blob_id, &range.map(|val| val.0)).await
}
