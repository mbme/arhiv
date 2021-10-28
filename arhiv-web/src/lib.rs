#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
#![allow(clippy::unused_async)]

use hyper::{Body, Request, Server, StatusCode};
use routerify::{ext::RequestExt, Middleware, Router, RouterService};

use arhiv_core::{entities::Id, prime_server::respond_with_attachment_data, Arhiv};
use public_assets::*;
use rs_utils::{
    log,
    server::{error_handler, logger_middleware, respond_with_status, ServerResponse},
};

mod public_assets;

pub async fn start_ui_server() {
    let arhiv = Arhiv::must_open();
    let port = arhiv.get_config().ui_server_port;

    let router = Router::builder()
        .data(arhiv)
        .middleware(Middleware::post_with_info(logger_middleware))
        .get("/public/:fileName", public_assets_handler)
        .get("/attachment-data/:id", attachment_data_handler)
        .post("/rpc", rpc_handler)
        .any(html5_index_handler)
        .err_handler_with_info(error_handler)
        .build()
        .expect("router must work");

    let service = RouterService::new(router).unwrap();

    let server = Server::bind(&(std::net::Ipv4Addr::LOCALHOST, port).into()).serve(service);
    let addr = server.local_addr();

    log::info!("UI server listening on http://{}", addr);

    if let Err(e) = server.with_graceful_shutdown(shutdown_signal()).await {
        log::error!("UI server error: {}", e);
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");

    println!();
    log::info!("Got Ctrl-C, stopping the server");
}

async fn attachment_data_handler(req: Request<Body>) -> ServerResponse {
    let id: Id = req.param("id").unwrap().as_str().into();

    let arhiv: &Arhiv = req.data().unwrap();

    respond_with_attachment_data(arhiv, &id).await
}

async fn html5_index_handler(_req: Request<Body>) -> ServerResponse {
    respond_with_status(StatusCode::OK)
}

async fn rpc_handler(_req: Request<Body>) -> ServerResponse {
    todo!()
}
