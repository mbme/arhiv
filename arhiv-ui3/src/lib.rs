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
use routerify::{Middleware, Router, RouterService};

use arhiv_core::Arhiv;
use pages::*;
use rs_utils::{
    log,
    server::{error_handler, logger_middleware, not_found_handler},
};

mod components;
mod markup;
mod pages;
mod ui_config;
mod urls;

#[macro_use]
mod utils;

pub async fn start_ui_server() {
    let arhiv = Arhiv::must_open();
    let port = arhiv.get_config().ui_server_port;

    let router = Router::builder()
        .data(arhiv)
        .middleware(Middleware::post_with_info(logger_middleware))
        .get("/public/:fileName", public_assets_handler)
        .get("/", index_page)
        //
        .get("/new", new_document_variants_page)
        .get("/new/:document_type", new_document_page)
        .post("/new/:document_type", new_document_page_handler)
        .get(
            "/collections/:collection_id/new/:document_type",
            new_document_page,
        )
        .post(
            "/collections/:collection_id/new/:document_type",
            new_document_page_handler,
        )
        //
        .get("/catalogs/:document_type", catalog_page)
        //
        .get("/documents/:id", document_page)
        .get("/collections/:collection_id/documents/:id", document_page)
        //
        .get("/documents/:id/edit", edit_document_page)
        .post("/documents/:id/edit", edit_document_page_handler)
        .get("/documents/:id/erase", erase_document_confirmation_dialog)
        .post(
            "/documents/:id/erase",
            erase_document_confirmation_dialog_handler,
        )
        //
        .get("/collections/:collection_id/documents/:id", document_page)
        .get(
            "/collections/:collection_id/documents/:id/edit",
            edit_document_page,
        )
        .post(
            "/collections/:collection_id/documents/:id/edit",
            edit_document_page_handler,
        )
        .get(
            "/collections/:collection_id/documents/:id/erase",
            erase_document_confirmation_dialog,
        )
        .post(
            "/collections/:collection_id/documents/:id/erase",
            erase_document_confirmation_dialog_handler,
        )
        //
        .get("/blobs/:blob_id", blob_handler)
        //
        .get("/modals/pick-document", pick_document_modal)
        .get("/modals/pick-file", pick_file_modal)
        .get(
            "/modals/pick-file-confirmation",
            pick_file_confirmation_modal,
        )
        .post(
            "/modals/pick-file-confirmation",
            pick_file_confirmation_modal_handler,
        )
        //
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        //
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
