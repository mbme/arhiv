use super::Arhiv;
use crate::entities::*;
use anyhow::*;
use rs_utils::read_file_as_stream;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use warp::{http, hyper, reply, Filter, Reply};

pub fn start_server(arhiv: Arc<Arhiv>) -> (JoinHandle<()>, oneshot::Sender<()>, SocketAddr) {
    let arhiv_filter = {
        let arhiv = arhiv.clone();

        warp::any().map(move || arhiv.clone())
    };

    // GET /status
    let get_status = warp::get()
        .and(warp::path("status"))
        .and(arhiv_filter.clone())
        .map(get_status_handler);

    // POST /attachment-data/:id file bytes
    let post_attachment_data = warp::post()
        .and(warp::path("attachment-data"))
        .and(warp::path::param::<String>())
        .and(warp::body::bytes())
        .and(arhiv_filter.clone())
        .map(post_attachment_data_handler);

    // GET /attachment-data/:id -> file bytes
    let get_attachment_data = warp::get()
        .and(warp::path("attachment-data"))
        .and(warp::path::param::<String>())
        .and(arhiv_filter.clone())
        .and_then(get_attachment_data_handler);

    // POST /changeset JSON Changeset -> JSON ChangesetResponse
    let post_changeset = warp::post()
        .and(warp::path("changeset"))
        .and(warp::body::json())
        .and(arhiv_filter.clone())
        .map(post_changeset_handler);

    let routes = get_status
        .or(post_attachment_data)
        .or(get_attachment_data)
        .or(post_changeset);

    let (shutdown_sender, shutdown_receiver) = oneshot::channel();

    let port = arhiv
        .config
        .get_server_port()
        .expect("config.server_port must be configured");
    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    log::info!("got Ctrl-C")
                }

                Ok(_) = shutdown_receiver => {
                    log::info!("got shutdown signal");
                }
            }
        });

    // Spawn the server into a runtime
    let join_handle = tokio::task::spawn(server);

    log::info!("started server on {}", addr);

    (join_handle, shutdown_sender, addr)
}

fn get_status_handler(arhiv: Arc<Arhiv>) -> impl warp::Reply {
    log::info!("Get arhiv status");

    let status = match arhiv.get_status() {
        Ok(status) => status,
        Err(err) => {
            log::error!("Failed to get arhiv status: {}", &err);

            return reply::with_status(
                format!("failed to get arhiv status: {}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    reply::with_status(status.to_string(), http::StatusCode::OK)
}

fn post_attachment_data_handler(
    id: String,
    data: warp::hyper::body::Bytes,
    arhiv: Arc<Arhiv>,
) -> impl warp::Reply {
    let id: Id = id.into();

    log::info!("Saving data for attachment {}", &id);

    let dst = arhiv.get_attachment_data(&id).get_staged_file_path();

    if Path::new(&dst).exists() {
        log::error!("temp attachment data {} already exists", dst);

        // FIXME check hashes instead of throwing an error
        return reply::with_status(
            format!("temp attachment data {} already exists", dst),
            http::StatusCode::CONFLICT,
        );
    }

    if let Err(err) = fs::write(dst, &data) {
        log::error!("Failed to save data for attachment {}: {}", &id, &err);

        return reply::with_status(
            format!("failed to write data: {}", err),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    reply::with_status("".to_string(), http::StatusCode::OK)
}

async fn get_attachment_data_handler(
    id: String,
    arhiv: Arc<Arhiv>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let id: Id = id.into();

    log::debug!("Serving data for attachment {}", &id);

    let attachment = match arhiv.get_document(&id) {
        Ok(Some(attachment)) if attachment.is_attachment() => attachment,

        Ok(Some(_)) => {
            log::warn!("Requested document {} isn't an attachment", &id);

            return Ok(reply::with_status(
                format!("Requested document {} isn't an attachment", &id),
                http::StatusCode::BAD_REQUEST,
            )
            .into_response());
        }

        Ok(None) => {
            log::warn!("Requested attachment {} is not found", &id);

            return Ok(reply::with_status(
                format!("can't find attachment with id {}", &id),
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }

        Err(err) => {
            log::error!("Failed to find attachment {}: {}", &id, &err);

            return Ok(reply::with_status(
                format!("failed to find attachment {}: {}", &id, err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    let path = arhiv.get_attachment_data(&id).get_committed_file_path();
    if !Path::new(&path).exists() {
        log::warn!("Requested attachment data {} is not found", &id);

        return Ok(reply::with_status(
            format!("can't find attachment data with id {}", &id),
            http::StatusCode::NOT_FOUND,
        )
        .into_response());
    }

    let file = match read_file_as_stream(&path).await {
        Ok(file) => file,
        Err(err) => {
            log::error!("Failed to read attachment data {}: {}", &id, &err);

            return Ok(reply::with_status(
                format!("failed to read attachment data {}: {}", &id, err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    // FIXME support ranges, status code: partial content
    // res.headers['Content-Type'] = await getMimeType(filePath)
    let filename = match arhiv.schema.get_field_string(&attachment, "filename") {
        Ok(filename) => filename,
        Err(err) => {
            log::error!("Failed to get attachment filename {}: {}", &id, &err);

            return Ok(reply::with_status(
                format!("failed to read attachment filename {}: {}", &id, err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    Ok(http::Response::builder()
        .header(
            "Content-Disposition",
            format!("inline; filename={}", filename),
        )
        .header("Cache-Control", "immutable, private, max-age=31536000") // max caching
        .body(hyper::Body::wrap_stream(file))
        .expect("must be able to construct response"))
}

fn post_changeset_handler(changeset: Changeset, arhiv: Arc<Arhiv>) -> impl warp::Reply {
    log::info!("Processing changeset {}", &changeset);

    match arhiv.has_staged_changes() {
        Ok(false) => {}
        Ok(true) => {
            log::error!("Rejecting changeset as arhiv has staged changes");

            return reply::with_status(
                "arhiv prime has staged changes",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
        Err(err) => {
            log::error!("Failed to check for staged changes: {:?}", err);

            return reply::with_status(
                format!("Failed to check for staged changes: {:?}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
    };

    let base_rev = changeset.base_rev.clone();

    if let Err(err) = arhiv.apply_changeset(changeset) {
        log::error!("Failed to apply a changeset: {:?}", err);

        return reply::with_status(
            format!("failed to apply a changeset: {:?}", err),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response();
    }

    match arhiv.generate_changeset_response(base_rev) {
        Ok(changeset_response) => {
            log::info!("Generated {}", &changeset_response);
            return reply::json(&changeset_response).into_response();
        }
        err => {
            log::error!("Failed to generate a changeset response: {:?}", err);

            return reply::with_status(
                format!("failed to apply a changeset: {:?}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response();
        }
    };
}
