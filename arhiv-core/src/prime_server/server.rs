use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use futures::TryStreamExt;
use hyper::HeaderMap;
use hyper::{body::Buf, http::request::Parts, Body, Request, Response, Server, StatusCode};
use routerify::{ext::RequestExt, Middleware, Router, RouterService};
use tokio::{signal, sync::oneshot, task::JoinHandle};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use rs_utils::{create_body_from_file, http_server::*, log, parse_range_header};

use crate::entities::{BLOBId, Changeset};
use crate::Arhiv;

#[must_use]
pub fn start_prime_server(
    arhiv: Arc<Arhiv>,
    port: u16,
) -> (JoinHandle<()>, oneshot::Sender<()>, SocketAddr) {
    let router = Router::builder()
        .data(arhiv)
        .middleware(Middleware::post_with_info(logger_middleware))
        .get("/status", status_handler)
        .get("/blobs/:blob_id", get_blob_handler)
        .post("/blobs/:blob_id", post_blob_handler)
        .post("/changeset", post_changeset_handler)
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        .build()
        .expect("router must work");

    let service = RouterService::new(router).expect("failed to build RouterService");

    let (shutdown_sender, shutdown_receiver) = oneshot::channel();

    let server = Server::bind(&(std::net::Ipv4Addr::UNSPECIFIED, port).into()).serve(service);

    let addr = server.local_addr();

    // Spawn the server into a runtime
    let join_handle = tokio::spawn(async move {
        server
            .with_graceful_shutdown(async {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        log::info!("got Ctrl-C");
                    }

                    Ok(_) = shutdown_receiver => {
                        log::info!("got shutdown signal");
                    }
                }
            })
            .await
            .expect("server must start");

        log::info!("started server on {}", addr);
    });

    (join_handle, shutdown_sender, addr)
}

#[allow(clippy::unused_async)]
async fn status_handler(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arc<Arhiv> = req.data().unwrap();

    let status = arhiv.get_status()?;

    json_response(status)
}

async fn post_blob_handler(req: Request<Body>) -> ServerResponse {
    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    let (parts, body): (Parts, Body) = req.into_parts();

    let arhiv: &Arc<Arhiv> = parts.data().unwrap();

    let blob = arhiv.get_blob(&blob_id)?;

    if blob.exists()? {
        return respond_with_status(StatusCode::CONFLICT);
    }

    let mut stream = body
        .map_ok(|mut buf| buf.copy_to_bytes(buf.remaining()))
        // Convert the stream into an futures::io::AsyncRead.
        // We must first convert the reqwest::Error into an futures::io::Error.
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = tokio::fs::File::create(&blob.file_path)
        .await
        .expect("must be able to create file");

    // Invoke tokio::io::copy to actually write file to disk
    if let Err(err) = tokio::io::copy(&mut stream, &mut file).await {
        log::error!("Failed to save blob {}: {}", &blob_id, &err);

        return respond_with_status(StatusCode::INTERNAL_SERVER_ERROR);
    }

    respond_with_status(StatusCode::OK)
}

async fn get_blob_handler(req: Request<Body>) -> ServerResponse {
    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    let arhiv: &Arc<Arhiv> = req.data().unwrap();

    respond_with_blob(arhiv, &blob_id, req.headers()).await
}

async fn post_changeset_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let changeset: Changeset = serde_json::from_slice(&body)?;

    let arhiv: &Arc<Arhiv> = parts.data().unwrap();

    let base_rev = changeset.base_rev;

    let mut tx = arhiv.get_tx()?;

    let conflicts = arhiv.apply_changeset(&mut tx, changeset)?;

    let response = tx.generate_changeset_response(base_rev, conflicts)?;

    tx.commit()?;

    json_response(response)
}

pub async fn respond_with_blob(
    arhiv: &Arhiv,
    blob_id: &BLOBId,
    headers: &HeaderMap,
) -> ServerResponse {
    let blob = arhiv.get_blob(blob_id)?;

    if !blob.exists()? {
        return respond_not_found();
    }

    let range_header = headers
        .get(hyper::header::RANGE)
        .map(|header| header.to_str())
        .transpose()
        .context("failed to convert HTTP Range header to string")?
        .map(|header| parse_range_header(header))
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
                format!("bytes {}-{}/{}", start_pos, end_pos, size),
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
