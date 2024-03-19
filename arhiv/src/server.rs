use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::http_server::{add_no_cache_headers, HttpServer};

use crate::{
    ui_server::{build_ui_router, UI_BASE_PATH},
    Arhiv, ArhivConfigExt,
};

pub struct ArhivServer {
    arhiv: Arc<Arhiv>,
    server: HttpServer,
    shutdown_sender: oneshot::Sender<()>,
}

impl ArhivServer {
    pub async fn start(arhiv: Arhiv) -> Result<Self> {
        let arhiv = Arc::new(arhiv);

        let conn = arhiv.baza.get_connection()?;
        let server_port = conn.get_server_port()?;
        let certificate = conn.get_certificate()?;

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let rpc_router = build_rpc_router(arhiv.baza.clone())?;
        let ui_router = build_ui_router(shutdown_receiver).with_state(arhiv.clone());

        let router = rpc_router
            .nest(UI_BASE_PATH, ui_router)
            .route("/health", get(health_handler));

        let server = HttpServer::new_https(server_port, router, certificate).await?;

        Ok(ArhivServer {
            arhiv,
            server,
            shutdown_sender,
        })
    }

    pub fn get_ui_url(&self) -> Result<String> {
        let url = self.server.get_url()?;

        Ok(format!("{url}{UI_BASE_PATH}"))
    }

    pub async fn shutdown(self) -> Result<()> {
        self.shutdown_sender
            .send(())
            .map_err(|_err| anyhow!("Arhiv Server shutdown receiver dropped"))?;

        self.server.shutdown().await?;

        Arc::into_inner(self.arhiv)
            .context("failed to unwrap Arhiv instance")?
            .stop();

        Ok(())
    }
}

#[allow(clippy::unused_async)]
async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}
