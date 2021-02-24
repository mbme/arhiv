use crate::commander::ArhivCommander;
use anyhow::*;
use arhiv::{get_attachment_data_handler, Arhiv};
use rs_utils::log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{signal, task::JoinHandle};
use warp::{path::Tail, Filter, Rejection, Reply};

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/static"]
struct Asset;

pub async fn start_server() -> (JoinHandle<()>, SocketAddr) {
    debug!("Assets:");
    for item in Asset::iter() {
        debug!("{}", item);
    }

    let arhiv = Arc::new(Arhiv::must_open());

    // POST /rpc RpcMessage -> RpcMessageResponse
    let rpc = {
        let commander = Arc::new(ArhivCommander::new(arhiv.clone()));
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
    let index_html = warp::path::end().and_then(serve_index);

    // GET /*
    let static_dir = warp::path::tail().and_then(serve);

    let routes = rpc.or(attachment_data).or(index_html).or(static_dir);

    // run server
    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
            signal::ctrl_c().await.expect("failed to listen for event");
            println!("\nGot Ctrl-C, stopping the server");
        });

    let join_handle = tokio::task::spawn(server);
    info!("RPC server listening on {}", addr);

    (join_handle, addr)
}

async fn serve_index() -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

async fn serve(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    debug!("GET {}", path);

    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = warp::reply::Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}

async fn rpc_action_handler(
    commander: Arc<ArhivCommander>,
    msg: RpcMessage,
) -> Result<impl Reply, Rejection> {
    debug!("RPC MESSAGE: {}", msg);

    let result = commander.run(msg.action, msg.params).await;

    let response = match result {
        Ok(result) => RpcMessageResponse { result, err: None },
        Err(err) => RpcMessageResponse {
            result: serde_json::Value::Null,
            err: Some(err.to_string()),
        },
    };

    Ok(warp::reply::json(&response).into_response())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessage {
    pub action: String,
    pub params: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessageResponse {
    pub result: Value,
    pub err: Option<String>,
}

impl fmt::Display for RpcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[RPC Message: {} {}]", self.action, self.params,)
    }
}
