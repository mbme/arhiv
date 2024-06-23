use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::StatusCode;
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::{
    http_server::{add_no_cache_headers, fallback_route, HttpServer},
    log,
};

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
    shutdown_sender: oneshot::Sender<()>,
}

impl ArhivServer {
    pub async fn start(root_dir: &str, options: ArhivOptions, server_port: u16) -> Result<Self> {
        let mut lock = ArhivServerLock::new(root_dir);
        lock.acquire()?;
        lock.write_server_port(server_port)?;

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let state = Arc::new(UIState::new(root_dir, options.clone(), shutdown_receiver)?);

        let certificate = read_or_generate_certificate(root_dir)?;
        let rpc_router = build_rpc_router(certificate.certificate_der.clone())?.route_layer(
            middleware::from_fn_with_state(state.clone(), extract_baza_from_state),
        );
        let ui_router = build_ui_router().with_state(state.clone());

        let router = Router::new()
            .merge(rpc_router)
            .nest(UI_BASE_PATH, ui_router)
            .route(HEALTH_PATH, get(health_handler))
            .fallback(fallback_route);

        let server = HttpServer::new_https(server_port, router, certificate).await?;

        let actual_server_port = server.get_address().port();
        lock.write_server_port(actual_server_port)?;

        if options.discover_peers {
            state.start_mdns_server(actual_server_port)?;
        }

        Ok(ArhivServer {
            state,
            server,
            shutdown_sender,
            _lock: lock,
        })
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

async fn extract_baza_from_state(
    State(state): State<Arc<UIState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let baza = match state.must_get_arhiv() {
        Ok(arhiv) => arhiv.baza.clone(),
        Err(err) => {
            log::error!("Attempt to access Arhiv that isn't initialized yet: {err}");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Arhiv not initialized: {err}"),
            )
                .into_response();
        }
    };

    request.extensions_mut().insert(baza);

    next.run(request).await
}

#[allow(clippy::unused_async)]
async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}
