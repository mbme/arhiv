#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::net::SocketAddr;

use hyper::Server;
use routerify::{Middleware, Router, RouterService};

use app_context::AppContext;
use arhiv_core::markup::RenderOptions;
use attachment_data::attachment_data_handler;
use pages::*;
use public_assets::*;
use rpc::*;
use rs_utils::{
    log,
    server::{error_handler, logger_middleware, not_found_handler},
};

mod app_context;
mod attachment_data;
mod components;
mod pages;
mod public_assets;
mod rpc;

pub async fn start_ui_server(port: u16) {
    let context = AppContext::new(RenderOptions {
        document_path: "/documents".to_string(),
        attachment_data_path: "/attachment-data".to_string(),
    })
    .expect("AppContext must init");

    let router = Router::builder()
        .data(context)
        .middleware(Middleware::post_with_info(logger_middleware))
        .get("/public/:fileName", public_assets_handler)
        .get("/", index_page)
        .get("/new", new_document_variants_page)
        .get("/new/:document_type", new_document_page)
        .get("/catalogs/:document_type", catalog_page)
        .get("/documents/:id", document_page)
        .get("/documents/:id/edit", document_editor_page)
        .get("/documents/:id/archive", archive_document_confirmation_page)
        .get("/documents/:id/delete", delete_document_confirmation_page)
        .get("/attachment-data/:hash", attachment_data_handler)
        .post("/rpc", rpc_handler)
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        .build()
        .expect("router must work");

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port));

    let server = Server::bind(&addr)
        .serve(service)
        .with_graceful_shutdown(shutdown_signal());

    log::info!("UI server listening on http://{}", addr);

    if let Err(e) = server.await {
        log::error!("UI server error: {}", e);
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");

    println!("");
    log::info!("Got Ctrl-C, stopping the server");
}
