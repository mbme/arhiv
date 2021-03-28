use crate::commander::ArhivCommander;
use crate::entities::*;
use crate::markup::RenderOptions;
use crate::Arhiv;
use anyhow::*;
use arhiv_ui_static_handler::*;
use futures::{Stream, TryStreamExt};
use rpc_handler::rpc_action_handler;
use rs_utils::log::{debug, error, info, warn};
use rs_utils::read_file_as_stream;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs as tokio_fs;
use tokio::signal;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use warp::{http, hyper, reply, Buf, Filter, Reply};

mod arhiv_ui_static_handler;
mod rpc_handler;

pub fn start_prime_server(arhiv: Arc<Arhiv>) -> (JoinHandle<()>, oneshot::Sender<()>, SocketAddr) {
    let arhiv_filter = {
        let arhiv = arhiv.clone();

        warp::any().map(move || arhiv.clone())
    };

    // GET /status
    let get_status = warp::get()
        .and(warp::path("status"))
        .and(arhiv_filter.clone())
        .map(get_status_handler);

    // POST /attachment-data/:hash file bytes
    let post_attachment_data = warp::post()
        .and(warp::path("attachment-data"))
        .and(warp::path::param::<String>())
        .and(warp::body::stream())
        .and(arhiv_filter.clone())
        .and_then(post_attachment_data_handler);

    // GET /attachment-data/:hash -> file bytes
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
        .and_then(post_changeset_handler);

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
                    info!("got Ctrl-C")
                }

                Ok(_) = shutdown_receiver => {
                    info!("got shutdown signal");
                }
            }
        });

    // Spawn the server into a runtime
    let join_handle = tokio::task::spawn(server);

    info!("started server on {}", addr);

    (join_handle, shutdown_sender, addr)
}

fn get_status_handler(arhiv: Arc<Arhiv>) -> impl warp::Reply {
    info!("Get arhiv status");

    let status = match arhiv.get_status() {
        Ok(status) => status,
        Err(err) => {
            error!("Failed to get arhiv status: {}", &err);

            return reply::with_status(
                format!("failed to get arhiv status: {}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    reply::with_status(status.to_string(), http::StatusCode::OK)
}

async fn post_attachment_data_handler(
    hash: String,
    data: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Unpin + 'static,
    arhiv: Arc<Arhiv>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let hash = Hash::from_string(hash);
    info!("Saving attachment data {}", &hash);

    let attachment_data = arhiv.get_attachment_data(hash);

    if attachment_data
        .exists()
        .expect("failed to check if attachment data exists")
    {
        warn!("attachment data {} already exists", attachment_data.path);

        // FIXME check hashes instead of throwing an error
        return Ok(reply::with_status(
            format!("attachment data {} already exists", attachment_data.path),
            http::StatusCode::CONFLICT,
        ));
    }

    let mut stream = data
        .map_ok(|mut buf| buf.copy_to_bytes(buf.remaining()))
        // Convert the stream into an futures::io::AsyncRead.
        // We must first convert the reqwest::Error into an futures::io::Error.
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = tokio_fs::File::create(&attachment_data.path)
        .await
        .expect("must be able to create file");

    // Invoke tokio::io::copy to actually write file to disk
    if let Err(err) = tokio::io::copy(&mut stream, &mut file).await {
        error!(
            "Failed to save attachment data {}: {}",
            &attachment_data.hash, &err
        );

        return Ok(reply::with_status(
            format!("failed to write data: {}", err),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    Ok(reply::with_status("".to_string(), http::StatusCode::OK))
}

async fn get_attachment_data_handler(
    hash: String,
    arhiv: Arc<Arhiv>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let hash = Hash::from_string(hash);
    debug!("Serving attachment data {}", &hash);

    let attachment_data = arhiv.get_attachment_data(hash);

    if !attachment_data
        .exists()
        .expect("failed to check if attachment data exists")
    {
        warn!(
            "Requested attachment data {} is not found",
            &attachment_data.hash
        );

        return Ok(reply::with_status(
            format!("can't find attachment data {}", &attachment_data.hash),
            http::StatusCode::NOT_FOUND,
        )
        .into_response());
    }

    let file = match read_file_as_stream(&attachment_data.path).await {
        Ok(file) => file,
        Err(err) => {
            error!(
                "Failed to read attachment data {}: {}",
                &attachment_data.hash, &err
            );

            return Ok(reply::with_status(
                format!(
                    "failed to read attachment data {}: {}",
                    &attachment_data.hash, err
                ),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    // FIXME support ranges, status code: partial content
    Ok(http::Response::builder()
        .header("Cache-Control", "immutable, private, max-age=31536000") // max caching
        .body(hyper::Body::wrap_stream(file))
        .expect("must be able to construct response"))
}

async fn post_changeset_handler(
    changeset: Changeset,
    arhiv: Arc<Arhiv>,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Processing changeset {}", &changeset);

    match arhiv.has_staged_changes() {
        Ok(false) => {}
        Ok(true) => {
            error!("Rejecting changeset as arhiv has staged changes");

            return Ok(reply::with_status(
                "arhiv prime has staged changes",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
        Err(err) => {
            error!("Failed to check for staged changes: {:?}", err);

            return Ok(reply::with_status(
                format!("Failed to check for staged changes: {:?}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    let base_rev = changeset.base_rev;

    if let Err(err) = arhiv.apply_changeset(changeset) {
        error!("Failed to apply a changeset: {:?}", err);

        return Ok(reply::with_status(
            format!("failed to apply a changeset: {:?}", err),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response());
    }

    match arhiv.generate_changeset_response(base_rev) {
        Ok(changeset_response) => {
            info!("Generated {}", &changeset_response);
            return Ok(reply::json(&changeset_response).into_response());
        }
        err => {
            error!("Failed to generate a changeset response: {:?}", err);

            return Ok(reply::with_status(
                format!("failed to apply a changeset: {:?}", err),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };
}

pub async fn start_ui_server() -> (JoinHandle<()>, SocketAddr) {
    let arhiv = Arc::new(Arhiv::must_open());

    // POST /rpc RpcMessage -> RpcMessageResponse
    let rpc = {
        let commander = Arc::new(ArhivCommander::new(
            arhiv.clone(),
            RenderOptions {
                document_path: "/document".to_string(),
                attachment_data_path: "/attachment-data".to_string(),
            },
        ));
        let commander_filter = warp::any().map(move || commander.clone());

        warp::path("rpc")
            .and(commander_filter)
            .and(warp::body::json())
            .and_then(rpc_action_handler)
    };

    // GET /attachment-data/:hash file bytes
    let attachment_data = {
        let arhiv = arhiv.clone();
        let arhiv_filter = warp::any().map(move || arhiv.clone());

        warp::path("attachment-data")
            .and(warp::path::param::<String>())
            .and(arhiv_filter.clone())
            .and_then(get_attachment_data_handler)
    };

    // GET / -> GET /index.html
    let index_html = warp::path::end().and_then(arhiv_ui_index_handler);

    // GET /*
    let static_dir = warp::path::tail().and_then(arhiv_ui_static_handler);

    let routes = rpc.or(attachment_data).or(index_html).or(static_dir);

    // run server
    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            signal::ctrl_c().await.expect("failed to listen for event");
            println!("\nGot Ctrl-C, stopping the server");
        });

    let join_handle = tokio::task::spawn(server);
    info!("RPC server listening on http://{}", addr);

    (join_handle, addr)
}
