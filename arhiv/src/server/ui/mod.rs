use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tokio::task::JoinHandle;
use warp::Filter;

use crate::commander::ArhivCommander;
use crate::markup::RenderOptions;
use crate::Arhiv;

use super::handlers::get_attachment_data_handler;
use arhiv_ui_static_handler::*;
use rpc_handler::rpc_action_handler;
use rs_utils::log;

mod arhiv_ui_static_handler;
mod rpc_handler;

pub async fn start_ui_server(port: Option<u16>) -> (JoinHandle<()>, SocketAddr) {
    let arhiv = Arc::new(Arhiv::must_open());

    // POST /rpc RpcMessage -> RpcMessageResponse
    let rpc = {
        let commander = Arc::new(ArhivCommander::new(
            arhiv.clone(),
            RenderOptions {
                document_path: "#/documents".to_string(),
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
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port.unwrap_or(0)), async {
            signal::ctrl_c().await.expect("failed to listen for event");

            println!("");
            log::info!("Got Ctrl-C, stopping the server");
        });

    let join_handle = tokio::task::spawn(server);
    log::info!("RPC server listening on http://{}", addr);

    (join_handle, addr)
}
