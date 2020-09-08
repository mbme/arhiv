use super::Arhiv;
use crate::entities::*;
use crate::storage::Queries;
use anyhow::*;
use bytes;
use rs_utils::read_file_as_stream;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use warp::{http, hyper, reply, Filter, Reply};

pub fn start_server<A: Into<Arc<Arhiv>>>(
    arhiv: A,
) -> (JoinHandle<()>, oneshot::Sender<()>, SocketAddr) {
    let arhiv = arhiv.into();

    let arhiv_filter = {
        let arhiv = arhiv.clone();

        warp::any().map(move || arhiv.clone())
    };

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

    let routes = post_attachment_data
        .or(get_attachment_data)
        .or(post_changeset);

    let (shutdown_sender, shutdown_receiver) = oneshot::channel();

    let port = arhiv.config.server_port;
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

impl Arhiv {
    fn exchange(&self, changeset: Changeset) -> Result<ChangesetResponse> {
        let (_, staged) = self.storage.get_connection()?.count_documents()?;
        if !changeset.is_empty() && staged > 0 {
            bail!("can't exchange: there are staged changes");
        }

        let base_rev = changeset.base_rev.clone();

        self.apply_changeset(changeset, false)?;

        self.generate_changeset_response(base_rev)
    }
}

fn post_attachment_data_handler(
    id: String,
    data: bytes::Bytes,
    arhiv: Arc<Arhiv>,
) -> impl warp::Reply {
    let dst = arhiv.storage.get_attachment_data(id).get_staged_file_path();

    if Path::new(&dst).exists() {
        log::error!("temp attachment data {} already exists", dst);

        // FIXME check hashes instead of throwing an error
        return reply::with_status(
            format!("temp attachment data {} already exists", dst),
            http::StatusCode::CONFLICT,
        );
    }

    if let Err(err) = fs::write(dst, &data) {
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
    let attachment = match arhiv.get_attachment(&id) {
        Ok(Some(attachment)) if attachment.is_committed() => attachment,

        Ok(Some(_)) | Ok(None) => {
            return Ok(reply::with_status(
                format!("can't find attachment with id {}", &id),
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }

        Err(err) => {
            return Ok(reply::with_status(
                format!("failed to find attachment {}: {}", &id, err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    let path = arhiv
        .storage
        .get_attachment_data(id.clone())
        .get_committed_file_path();
    if !Path::new(&path).exists() {
        return Ok(reply::with_status(
            format!("can't find attachment with id {}", &id),
            http::StatusCode::NOT_FOUND,
        )
        .into_response());
    }

    let file = match read_file_as_stream(&path).await {
        Ok(file) => file,
        Err(err) => {
            return Ok(reply::with_status(
                format!("failed to read attachment {}: {}", &id, err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    // FIXME support ranges, status code: partial content
    // res.headers['Content-Type'] = await getMimeType(filePath)

    Ok(http::Response::builder()
        .header(
            "Content-Disposition",
            format!("inline; filename={}", attachment.filename),
        )
        .header("Cache-Control", "immutable, private, max-age=31536000") // max caching
        .body(hyper::Body::wrap_stream(file))
        .expect("must be able to construct response"))
}

fn post_changeset_handler(changeset: Changeset, arhiv: Arc<Arhiv>) -> impl warp::Reply {
    let result = arhiv.exchange(changeset);

    match result {
        Ok(changeset_response) => reply::json(&changeset_response).into_response(),
        err => {
            log::error!("Failed to apply a changeset: {:?}", err);

            reply::with_status(
                format!("failed to apply a changeset: {:?}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}
