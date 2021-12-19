use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use futures::TryStreamExt;
use hyper::{body::Buf, http::request::Parts, Body, Request, Response, Server, StatusCode};
use routerify::{ext::RequestExt, Middleware, Router, RouterService};
use tokio::{signal, sync::oneshot, task::JoinHandle};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use rs_utils::{log, read_file_as_stream, server::*};

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
    let join_handle = tokio::task::spawn(async move {
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
async fn status_handler(req: Request<Body>) -> Result<Response<Body>> {
    let arhiv: &Arc<Arhiv> = req.data().unwrap();

    let status = arhiv.get_status()?;

    json_response(status)
}

async fn post_blob_handler(req: Request<Body>) -> Result<Response<Body>> {
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

async fn get_blob_handler(req: Request<Body>) -> Result<Response<Body>> {
    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    let arhiv: &Arc<Arhiv> = req.data().unwrap();

    respond_with_blob(arhiv, &blob_id).await
}

async fn post_changeset_handler(req: Request<Body>) -> Result<Response<Body>> {
    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let changeset: Changeset = serde_json::from_slice(&body)?;

    let arhiv: &Arc<Arhiv> = parts.data().unwrap();

    let base_rev = changeset.base_rev;

    let mut tx = arhiv.db.get_tx()?;

    let conflicts = arhiv.apply_changeset(&mut tx, changeset)?;

    let response = arhiv.generate_changeset_response(&tx, base_rev, conflicts)?;

    tx.commit()?;

    json_response(response)
}

pub async fn respond_with_blob(arhiv: &Arhiv, blob_id: &BLOBId) -> ServerResponse {
    let blob = arhiv.get_blob(blob_id)?;

    if !blob.exists()? {
        return respond_not_found();
    }

    let file = read_file_as_stream(&blob.file_path).await?;

    Response::builder()
        .header(
            // max caching
            hyper::header::CACHE_CONTROL,
            "immutable, private, max-age=31536000",
        )
        .body(Body::wrap_stream(file))
        .context("failed to build response")
}
