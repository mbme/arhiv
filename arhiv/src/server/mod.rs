use std::sync::Arc;

use anyhow::Result;
use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use certificate::generate_ui_crypto_key;

use rs_utils::{
    create_dir_if_not_exist,
    http_server::{add_no_cache_headers, fallback_route, HttpServer},
    log,
};

use self::{
    certificate::read_or_generate_certificate,
    server_lock::ArhivServerLock,
    ui_server::{build_ui_router, UI_BASE_PATH},
};
use crate::{Arhiv, ArhivOptions};

pub use self::server_info::ServerInfo;

mod certificate;
mod server_info;
mod server_lock;
mod ui_server;

pub const HEALTH_PATH: &str = "/health";

pub struct ArhivServer {
    pub arhiv: Arc<Arhiv>,
    server: HttpServer,
    _lock: ArhivServerLock,
}

impl ArhivServer {
    pub async fn start(options: ArhivOptions, server_port: u16) -> Result<Self> {
        let state_dir = options.state_dir.clone();
        log::info!("Starting server in {state_dir}");

        create_dir_if_not_exist(&state_dir)?;

        let mut lock = ArhivServerLock::new(&state_dir);
        lock.acquire()?;
        lock.write_server_port(server_port)?;

        let mut arhiv = Arhiv::new(options);
        arhiv.init_auto_commit_service();

        let arhiv = Arc::new(arhiv);

        let certificate = read_or_generate_certificate(&state_dir)?;

        let ui_key = generate_ui_crypto_key(certificate.private_key_der.clone())?;
        let ui_router = build_ui_router(ui_key).with_state(arhiv.clone());

        let router = Router::new()
            .nest(UI_BASE_PATH, ui_router)
            .route(HEALTH_PATH, get(health_handler))
            .fallback(fallback_route);

        let server = HttpServer::new_https(server_port, router, certificate).await?;

        let actual_server_port = server.get_address().port();
        lock.write_server_port(actual_server_port)?;

        Ok(ArhivServer {
            arhiv,
            server,
            _lock: lock,
        })
    }

    pub async fn shutdown(self) -> Result<()> {
        self.server.shutdown().await?;

        self.arhiv.stop();

        Ok(())
    }
}

#[allow(clippy::unused_async)]
async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}
