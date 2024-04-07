use std::{fs, sync::Arc};

use anyhow::{anyhow, Context, Result};
use axum::Router;
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::{
    file_in_temp_dir,
    http_server::{fallback_route, HttpServer},
    LockFile,
};

use crate::{
    ui_server::{build_ui_router, UIState, UI_BASE_PATH},
    Arhiv, ArhivOptions,
};

struct ArhivServerLock {
    lock_file: String,
    lock: Option<LockFile>,
}

impl ArhivServerLock {
    pub fn new() -> Self {
        let lock_file = file_in_temp_dir("arhiv-server.lock");

        Self {
            lock_file,
            lock: None,
        }
    }

    pub fn acquire(&mut self) -> Result<()> {
        self.lock = Some(LockFile::new(&self.lock_file)?);

        Ok(())
    }

    pub fn read_server_port(&self) -> Result<u16> {
        let value = fs::read_to_string(&self.lock_file)?;

        value
            .trim()
            .parse::<u16>()
            .with_context(|| format!("Can't parse port number from string `{}`", value))
    }

    pub fn write_server_port(&self, port: u16) -> Result<()> {
        fs::write(&self.lock_file, port.to_string())
            .context("Failed to write server port into lock file")
    }
}

pub struct ArhivServer {
    state: Arc<UIState>,
    server: HttpServer,
    _lock: ArhivServerLock,
    shutdown_sender: oneshot::Sender<()>,
}

impl ArhivServer {
    pub async fn start(
        root_dir: &str,
        mut options: ArhivOptions,
        server_port: u16,
    ) -> Result<Self> {
        let mut lock = ArhivServerLock::new();
        lock.acquire()?;

        let certificate = options
            .certificate
            .unwrap_or_else(Arhiv::generate_certificate);
        options.certificate = Some(certificate.clone());

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        let state = Arc::new(UIState::new(root_dir, options.clone())?);

        let rpc_router = build_rpc_router(certificate.certificate_der.clone())?;
        let ui_router = build_ui_router(certificate.certificate_der.clone(), shutdown_receiver)
            .with_state(state.clone());

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

    pub fn get_server_port() -> Result<Option<u16>> {
        let mut lock = ArhivServerLock::new();

        // server isn't running
        if lock.acquire().is_ok() {
            return Ok(None);
        }

        lock.read_server_port().map(Some)
    }
}
