use std::sync::Arc;

use anyhow::Context;
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use routerify::{ext::RequestExt, Middleware, Router};

use rs_utils::{create_body_from_file, http_server::*, log, parse_range_header};

use crate::entities::BLOBId;
use crate::sync::Revision;
use crate::Baza;

pub fn build_rpc_router(
    baza: Arc<Baza>,
    ui_router: Option<Router<Body, anyhow::Error>>,
) -> Router<Body, anyhow::Error> {
    let mut builder = Router::builder()
        .data(baza)
        .middleware(Middleware::post_with_info(logger_middleware))
        //
        .get("/health", health_handler)
        .get("/blobs/:blob_id", get_blob_handler)
        .get("/ping", get_ping_handler)
        .get("/changeset/:min_rev", get_changeset_handler);

    if let Some(ui_router) = ui_router {
        builder = builder.scope("/ui", ui_router);
    }

    builder
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        //
        .build()
        .expect("router must work")
}

pub fn start_rpc_server(baza: Arc<Baza>, port: u16) -> HttpServer {
    let router = build_rpc_router(baza, None);

    HttpServer::start(router, port)
}

#[allow(clippy::unused_async)]
async fn health_handler(_req: Request<Body>) -> ServerResponse {
    respond_with_status(StatusCode::OK)
}

async fn get_blob_handler(req: Request<Body>) -> ServerResponse {
    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    let baza: &Arc<Baza> = req.data().unwrap();

    respond_with_blob(baza, &blob_id, req.headers()).await
}

async fn get_ping_handler(req: Request<Body>) -> ServerResponse {
    let baza: &Arc<Baza> = req.data().unwrap();

    let ping = baza.get_connection()?.get_ping()?;

    json_response(ping)
}

async fn get_changeset_handler(req: Request<Body>) -> ServerResponse {
    let min_rev = req.param("min_rev").unwrap().as_str();
    let min_rev = serde_json::from_str(min_rev).context("failed to parse min_rev")?;
    let min_rev = Revision::from_value(min_rev)?;

    let baza: &Arc<Baza> = req.data().unwrap();

    let changeset = baza.get_connection()?.get_changeset(&min_rev)?;

    json_response(changeset)
}

pub async fn respond_with_blob(
    baza: &Baza,
    blob_id: &BLOBId,
    headers: &HeaderMap,
) -> ServerResponse {
    let conn = baza.get_connection()?;
    let blob = conn.get_blob(blob_id);

    if !blob.exists()? {
        return respond_not_found();
    }

    let range_header = headers
        .get(hyper::header::RANGE)
        .map(hyper::header::HeaderValue::to_str)
        .transpose()
        .context("failed to convert HTTP Range header to string")?
        .map(parse_range_header)
        .transpose()
        .context("failed to parse HTTP Range header")?;

    let size = blob.get_size()?;

    let response_builder = Response::builder()
        .header(
            // max caching
            hyper::header::CACHE_CONTROL,
            "immutable, private, max-age=31536000",
        )
        .header(hyper::header::CONTENT_TYPE, blob.get_media_type()?)
        .header(hyper::header::ACCEPT_RANGES, "bytes");

    if let Some((start_pos, end_pos)) = range_header {
        let end_pos = end_pos.unwrap_or(size - 1);

        if start_pos >= size || start_pos >= end_pos || end_pos >= size {
            log::warn!(
                "blob {}: range {}-{} out of {} not satisfiable",
                blob_id,
                start_pos,
                end_pos,
                size
            );
            return respond_with_status(StatusCode::RANGE_NOT_SATISFIABLE);
        }

        let range_size = end_pos + 1 - start_pos;

        let body = create_body_from_file(&blob.file_path, start_pos, Some(range_size)).await?;

        response_builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(hyper::header::CONTENT_LENGTH, range_size)
            .header(
                hyper::header::CONTENT_RANGE,
                format!("bytes {start_pos}-{end_pos}/{size}"),
            )
            .body(body)
            .context("failed to build BLOB response")
    } else {
        let body = create_body_from_file(&blob.file_path, 0, None).await?;

        response_builder
            .status(StatusCode::OK)
            .header(hyper::header::CONTENT_LENGTH, size)
            .body(body)
            .context("failed to build BLOB response")
    }
}
