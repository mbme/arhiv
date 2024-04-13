use std::{fs, io::Write, sync::Arc};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use reqwest::StatusCode;
use tokio::sync::oneshot;

use baza::{sync::build_rpc_router, DEV_MODE};
use rs_utils::{
    file_exists,
    http_server::{fallback_route, HttpServer},
    log, must_create_file, now, LockFile, SecretBytes, SecretString, SelfSignedCertificate,
};

use crate::{
    ui_server::{build_ui_router, UIState, UI_BASE_PATH},
    ArhivOptions,
};

struct ArhivServerLock {
    lock_file: String,
    lock: Option<LockFile>,
}

impl ArhivServerLock {
    pub fn new(root_dir: &str) -> Self {
        let file_name = if DEV_MODE {
            "arhiv-server-dev.lock"
        } else {
            "arhiv-server.lock"
        };

        let lock_file = format!("{root_dir}/{file_name}");
        log::debug!("Arhiv server lock file: {lock_file}");

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
    pub async fn start(root_dir: &str, options: ArhivOptions, server_port: u16) -> Result<Self> {
        let mut lock = ArhivServerLock::new(root_dir);
        lock.acquire()?;

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

fn read_or_generate_certificate(root_dir: &str) -> Result<SelfSignedCertificate> {
    let cert_path = format!("{root_dir}/certificate.pfx");
    let password = SecretString::new("");

    if file_exists(&cert_path)? {
        let data = fs::read(&cert_path).context("Failed to read certificate file")?;

        let data = SecretBytes::new(data);

        let certificate = SelfSignedCertificate::from_pfx_der(&password, data)?;

        log::info!("Read arhiv certificate from {cert_path}");

        Ok(certificate)
    } else {
        let certificate = generate_certificate()?;

        let friendly_name = if DEV_MODE { "arhiv-dev" } else { "arhiv" };

        let data = certificate.to_pfx_der(&password, friendly_name)?;

        // Save Arhiv's certificate in PKCS#12 format (.pfx). Browsers can use it as a client HTTPS/TLS certificate. Password is empty.
        let mut file = must_create_file(&cert_path)
            .context(anyhow!("Failed to create certificate file {cert_path}"))?;
        file.write_all(data.as_bytes())?;
        file.flush()?;

        log::info!("Wrote arhiv certificate into {cert_path}");

        Ok(certificate)
    }
}

fn generate_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}
