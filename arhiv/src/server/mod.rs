use std::sync::Arc;

use anyhow::Result;

use rs_utils::{create_dir_if_not_exist, http_server::HttpServer, log};

use self::{
    certificate::read_or_generate_certificate, server_lock::ArhivServerLock,
    ui_server::build_ui_router,
};
use crate::{Arhiv, ArhivOptions};

pub use self::server_info::ServerInfo;

mod certificate;
mod server_info;
mod server_lock;
mod ui_server;

pub struct ArhivServer {
    pub arhiv: Arc<Arhiv>,
    server: HttpServer,
    _lock: ArhivServerLock,
    server_info: ServerInfo,
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

        let router = build_ui_router(&certificate, arhiv.clone());

        let server = HttpServer::new_https(server_port, router, certificate.clone()).await?;

        let actual_server_port = server.get_address().port();
        lock.write_server_port(actual_server_port)?;

        log::info!("Started server on port: {actual_server_port}");

        let server_info = ServerInfo::new(actual_server_port, &certificate);

        Ok(ArhivServer {
            arhiv,
            server,
            _lock: lock,
            server_info,
        })
    }

    pub async fn shutdown(self) -> Result<()> {
        self.server.shutdown().await?;

        self.arhiv.stop();

        Ok(())
    }

    pub fn get_info(&self) -> &ServerInfo {
        &self.server_info
    }
}
