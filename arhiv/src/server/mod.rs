use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use reqwest::StatusCode;
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::{
    http_server::{fallback_route, HttpServer},
    log,
};

use self::ui_server::{build_ui_router, UIState};
use self::{certificate::read_or_generate_certificate, server_lock::ArhivServerLock};
use crate::ArhivOptions;

pub use self::ui_server::UI_BASE_PATH;

mod certificate;
mod server_lock;
mod ui_server;

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

    pub fn get_server_port(root_dir: &str) -> Result<Option<u16>> {
        let mut lock = ArhivServerLock::new(root_dir);

        // server isn't running
        if lock.acquire().is_ok() {
            return Ok(None);
        }

        lock.read_server_port().map(Some)
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
