use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::http_server::{add_no_cache_headers, HttpServer};

use crate::{
    ui_server::{build_ui_router, UIState, UI_BASE_PATH},
    ArhivOptions,
};

pub struct ArhivServer {
    state: Arc<UIState>,
    server: HttpServer,
    shutdown_sender: oneshot::Sender<()>,
}

impl ArhivServer {
    pub async fn start(root_dir: &str, options: ArhivOptions) -> Result<Self> {
        let server_port = 0; // FIXME

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let state = Arc::new(UIState::new(root_dir, options)?);

        let rpc_router = build_rpc_router(state.get_certificate().certificate_der.clone())?;
        let ui_router = build_ui_router(shutdown_receiver).with_state(state.clone());

        let router = rpc_router
            .nest(UI_BASE_PATH, ui_router)
            .route("/health", get(health_handler));

        let server =
            HttpServer::new_https(server_port, router, state.get_certificate().clone()).await?;

        Ok(ArhivServer {
            state,
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

        self.state.stop_arhiv()?;

        Ok(())
    }
}

#[allow(clippy::unused_async)]
async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}
