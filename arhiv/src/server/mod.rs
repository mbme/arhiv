use std::sync::Arc;

use anyhow::Result;
use axum::{http::HeaderMap, response::IntoResponse, routing::get, Router};
use certificate::generate_ui_crypto_key;
use reqwest::StatusCode;

use rs_utils::http_server::{add_no_cache_headers, fallback_route, HttpServer};

use self::ui_server::{build_ui_router, UIState, UI_BASE_PATH};
use self::{certificate::read_or_generate_certificate, server_lock::ArhivServerLock};
use crate::ArhivOptions;

pub use self::server_info::ServerInfo;

mod certificate;
mod server_info;
mod server_lock;
mod ui_server;

pub const HEALTH_PATH: &str = "/health";

pub struct ArhivServer {
    state: Arc<UIState>,
    server: HttpServer,
    _lock: ArhivServerLock,
}

impl ArhivServer {
    pub async fn start(root_dir: &str, options: ArhivOptions, server_port: u16) -> Result<Self> {
        let mut lock = ArhivServerLock::new(root_dir);
        lock.acquire()?;
        lock.write_server_port(server_port)?;

        let state = Arc::new(UIState::new(root_dir, options.clone())?);

        let certificate = read_or_generate_certificate(root_dir)?;

        let ui_key = generate_ui_crypto_key(certificate.private_key_der.clone())?;
        let ui_router = build_ui_router(ui_key).with_state(state.clone());

        let router = Router::new()
            .nest(UI_BASE_PATH, ui_router)
            .route(HEALTH_PATH, get(health_handler))
            .fallback(fallback_route);

        let server = HttpServer::new_https(server_port, router, certificate).await?;

        let actual_server_port = server.get_address().port();
        lock.write_server_port(actual_server_port)?;

        Ok(ArhivServer {
            state,
            server,
            _lock: lock,
        })
    }

    pub async fn shutdown(self) -> Result<()> {
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
