use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use tokio::sync::oneshot;

use baza::sync::build_rpc_router;
use rs_utils::http_server::{build_health_router, HttpServer};

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

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let health_router = build_health_router();
        let rpc_router = build_rpc_router();
        let ui_router = build_ui_router(shutdown_receiver);

        let router = rpc_router
            .nest(UI_BASE_PATH, ui_router.with_state(arhiv.clone()))
            .with_state(arhiv.baza.clone())
            .merge(health_router);

        let port = arhiv.baza.get_connection()?.get_server_port()?;
        let server = HttpServer::new_http(port, router).await?;

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
