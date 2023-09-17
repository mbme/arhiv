use anyhow::Context;
use hyper::{header, http::request::Parts, Body, Request, Response, Server, StatusCode};
use routerify::{prelude::RequestExt, Middleware, Router, RouterService};

use arhiv_core::Arhiv;
use baza::{entities::BLOBId, sync::respond_with_blob};
use public_assets_handler::public_assets_handler;
use rs_utils::{
    http_server::{
        error_handler, logger_middleware, not_found_handler, respond_moved_permanently,
        respond_with_status, ServerResponse,
    },
    log,
};

use crate::dto::APIRequest;

use api_handler::handle_api_request;

mod api_handler;
mod public_assets_handler;

pub async fn start_ui_server() {
    let arhiv = Arhiv::must_open();
    let port = arhiv.get_config().ui_server_port;

    let router = Router::builder()
        .data(arhiv)
        .middleware(Middleware::post_with_info(logger_middleware))
        //
        .get("/", index_page)
        .post("/api", api_handler)
        .get("/documents/:document_id", old_document_page_handler) // redirect for compatibility with the old UI
        .get("/blobs/:blob_id", blob_handler)
        .get("/:fileName", public_assets_handler)
        //
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        //
        .build()
        .expect("failed to build router");

    let service = RouterService::new(router).expect("failed to build router service");

    let server = Server::bind(&(std::net::Ipv4Addr::LOCALHOST, port).into()).serve(service);
    let addr = server.local_addr();

    log::info!("UI server listening on http://{}", addr);

    if let Err(e) = server.with_graceful_shutdown(shutdown_signal()).await {
        log::error!("UI server error: {}", e);
    }
}

async fn index_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let schema =
        serde_json::to_string(arhiv.baza.get_schema()).context("failed to serialize schema")?;

    let content = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en" dir="ltr">
                <head>
                    <title>Arhiv</title>

                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                    <link rel="stylesheet" href="/index.css" />
                </head>
                <body>
                    <main></main>

                    <script>
                        window.SCHEMA = {schema};
                    </script>

                    <script src="/index.js"></script>
                </body>
            </html>"#
    );

    render_html(StatusCode::OK, content)
}

async fn old_document_page_handler(req: Request<Body>) -> ServerResponse {
    let document_id = req.param("document_id").unwrap().as_str();

    respond_moved_permanently(format!("/?id={document_id}"))
}

async fn api_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let arhiv: &Arhiv = parts.data().unwrap();

    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str())
        .transpose()?
        .unwrap_or_default();

    if content_type != "application/json" {
        return respond_with_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    let body = hyper::body::to_bytes(body).await?;

    let request: APIRequest =
        serde_json::from_slice(&body).context("failed to parse api request")?;

    let response = handle_api_request(arhiv, request).await?;

    let content = serde_json::to_string(&response).context("failed to serialize response")?;

    render_json(StatusCode::OK, content)
}

async fn blob_handler(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&arhiv.baza, &blob_id, req.headers()).await
}

fn build_response(status: StatusCode, content_type: &str, content: String) -> ServerResponse {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        // prevent page from caching
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::EXPIRES, "0")
        // ---
        .body(content.into())
        .context("failed to build response")
}

fn render_html(status: StatusCode, content: String) -> ServerResponse {
    build_response(status, "text/html", content)
}

fn render_json(status: StatusCode, content: String) -> ServerResponse {
    build_response(status, "application/json", content)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");

    println!();
    log::info!("Got Ctrl-C, stopping the server");
}
