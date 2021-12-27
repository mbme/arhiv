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

use hyper::Server;

use arhiv_core::Arhiv;
use rs_utils::log;

use crate::{app::App, routes::build_router_service};

mod app;
mod components;
mod markup;
mod pages;
mod public_assets_handler;
mod routes;
mod ui_config;
mod urls;

#[macro_use]
mod utils;

pub async fn start_ui_server() {
    let arhiv = Arhiv::must_open();
    let port = arhiv.get_config().ui_server_port;
    let app = App::new(arhiv);

    let service = build_router_service(app).expect("router must work");

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
