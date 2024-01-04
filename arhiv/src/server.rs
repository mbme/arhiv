use std::sync::Arc;

use anyhow::{Context, Result};

use baza::sync::build_rpc_router;
use rs_utils::http_server::{build_health_router, HttpServer};

use crate::{ui_server::build_ui_router, Arhiv, ArhivConfigExt};

pub struct ArhivServer {
    arhiv: Arc<Arhiv>,
    server: HttpServer,
}

impl ArhivServer {
    pub fn start(arhiv: Arhiv) -> Result<Self> {
        let arhiv = Arc::new(arhiv);

        let health_router = build_health_router();
        let rpc_router = build_rpc_router();
        let ui_router = build_ui_router();

        let router = rpc_router
            .nest("/ui", ui_router.with_state(arhiv.clone()))
            .with_state(arhiv.baza.clone())
            .merge(health_router);

        let port = arhiv.baza.get_connection()?.get_server_port()?;
        let server = HttpServer::start(router, port);

        Ok(ArhivServer { arhiv, server })
    }

    pub fn get_ui_url(&self) -> String {
        format!("http://{}/ui", self.server.get_address())
    }

    pub async fn join(self) -> Result<()> {
        self.server.join().await?;

        Arc::into_inner(self.arhiv)
            .context("failed to unwrap Arhiv instance")?
            .stop();

        Ok(())
    }

    pub async fn stop(self) -> Result<()> {
        self.server.shutdown().await?;

        Arc::into_inner(self.arhiv)
            .context("failed to unwrap Arhiv instance")?
            .stop();

        Ok(())
    }
}
