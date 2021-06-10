use std::net::SocketAddr;

use anyhow::*;
use hyper::{Body, Response, Server, StatusCode};
use routerify::{Middleware, RequestInfo, Router, RouterService};

use app_context::AppContext;
use arhiv::markup::RenderOptions;
use http_utils::not_found;
use pages::*;
use public_assets::*;
use rpc::*;
use rs_utils::log::{self, setup_logger};

mod app_context;
mod components;
mod http_utils;
mod pages;
mod public_assets;
mod rpc;

#[tokio::main]
async fn main() {
    setup_logger();

    let context = AppContext::new(RenderOptions {
        document_path: "/documents".to_string(),
        attachment_data_path: "/attachment-data".to_string(),
    })
    .expect("AppContext must init");

    let router = Router::builder()
        .data(context)
        .middleware(Middleware::post_with_info(logger))
        .get("/public/:fileName", public_assets_handler)
        .get("/", index_page)
        .get("/new", new_document_page)
        .get("/new/:document_type", new_document_page)
        .get("/catalogs/:document_type", catalog_page)
        .get("/documents/:id", document_page)
        .get("/documents/:id/edit", document_editor_page)
        .get("/documents/:id/archive", archive_document_confirmation_page)
        .get("/documents/:id/delete", delete_document_confirmation_page)
        .post("/rpc", rpc_handler)
        .any(|_| async { not_found() })
        .err_handler_with_info(error_handler)
        .build()
        .expect("router must work");

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let server = Server::bind(&addr)
        .serve(service)
        .with_graceful_shutdown(shutdown_signal());

    log::info!("running on: {}", addr);

    if let Err(err) = server.await {
        log::error!("Server error: {}", err);
    }
}

async fn logger(res: Response<Body>, info: RequestInfo) -> Result<Response<Body>> {
    log::info!(
        "{} {} -> {}",
        info.method(),
        info.uri().path(),
        res.status()
    );

    Ok(res)
}

async fn error_handler(err: routerify::RouteError, info: RequestInfo) -> Response<Body> {
    log::error!("{} {} -> {}", info.method(), info.uri().path(), err);

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
