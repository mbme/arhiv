use hyper::Server;

use arhiv_core::Arhiv;
use rs_utils::log;

use app::App;
use routes::build_router_service;

mod app;
mod public_assets_handler;
mod routes;
mod workspace_api_handler;
mod workspace_page;

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
