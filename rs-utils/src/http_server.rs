use std::{
    collections::HashSet,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{anyhow, ensure, Context, Result};
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::headers::{self, HeaderMapExt};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use reqwest::Client;
use rustls::{
    server::{ClientCertVerifier, NoClientAuth},
    Certificate, PrivateKey, ServerConfig,
};
use tokio::task::JoinHandle;

pub struct ServerError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        log::error!("server error: {}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong:\n{:?}", self.0),
        )
            .into_response()
    }
}

// make ServerError compatible with anyhow::Error
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// prevent page from caching
pub fn add_no_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    headers.typed_insert(headers::Expires::from(UNIX_EPOCH));
}

pub fn add_max_cache_header(headers: &mut HeaderMap) {
    headers.typed_insert(
        headers::CacheControl::new()
            .with_immutable()
            .with_private()
            .with_max_age(Duration::from_secs(31536000)),
    );
}

#[allow(clippy::unused_async)]
async fn health_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_no_cache_headers(&mut headers);

    (StatusCode::OK, headers)
}

pub fn build_health_router() -> Router<()> {
    Router::new().route("/health", get(health_handler))
}

pub async fn check_server_health(server_url: &str) -> Result<()> {
    let url = reqwest::Url::from_str(&format!("https://{server_url}/health"))
        .context("failed to create url from server address")?;

    let response = Client::new().get(url).send().await?;

    let status = response.status();

    ensure!(
        status.is_success(),
        "expected status code 2xx, got {}",
        status.to_string()
    );

    Ok(())
}

pub async fn logger_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let method = request.method().to_string();
    let uri = request.uri().path().to_string();

    let response = next.run(request).await;

    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let (parts, body) = response.into_parts();

        let bytes = to_bytes(body, 10_000_000)
            .await
            .context("failed to convert body to bytes")
            .unwrap_or_default();

        let body_str = String::from_utf8_lossy(&bytes);
        log::warn!("{method} {uri} -> {status}:\n{body_str}");

        Ok(Response::from_parts(parts, Body::from(bytes)))
    } else {
        log::debug!("{method} {uri} -> {status}");
        Ok(response)
    }
}

pub struct HttpServer {
    address: SocketAddr,
    join_handle: JoinHandle<Result<()>>,
    server_handle: Handle,
    secure: bool,
}

impl HttpServer {
    pub async fn new_http(port: u16, router: Router) -> Result<Self> {
        let addr: SocketAddr = (std::net::Ipv4Addr::UNSPECIFIED, port).into();
        let server_handle = Handle::new();
        let server = axum_server::Server::bind(addr).handle(server_handle.clone());

        // Spawn the server into a runtime
        let join_handle = tokio::spawn(async {
            let router = router.layer(middleware::from_fn(logger_middleware));

            server
                .serve(router.into_make_service())
                .await
                .context("HTTP Server failed to start")?;

            log::info!("HTTP Server exited");

            Ok(())
        });

        let address = server_handle
            .listening()
            .await
            .context("HTTP Server failed to bind address")?;

        log::info!("HTTP Server started on {}", address);

        Ok(HttpServer {
            server_handle,
            join_handle,
            address,
            secure: false,
        })
    }

    pub async fn new_https(
        port: u16,
        router: Router,
        server_cert: Vec<u8>,
        server_private_key: Vec<u8>,
        client_cert_verifier: Option<Arc<dyn ClientCertVerifier>>,
    ) -> Result<Self> {
        let certificate = Certificate(server_cert);
        let private_key = PrivateKey(server_private_key);
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(
                client_cert_verifier.unwrap_or_else(|| NoClientAuth::boxed()),
            )
            .with_single_cert(vec![certificate], private_key)?;

        let config = RustlsConfig::from_config(Arc::new(config));

        let server_handle = Handle::new();

        let addr: SocketAddr = (std::net::Ipv4Addr::UNSPECIFIED, port).into();
        let server = axum_server::bind_rustls(addr, config).handle(server_handle.clone());

        // Spawn the server into a runtime
        let join_handle = tokio::spawn(async {
            let router = router.layer(middleware::from_fn(logger_middleware));

            server
                .serve(router.into_make_service())
                .await
                .context("HTTPS Server failed to start")?;

            log::info!("HTTPS Server exited");

            Ok(())
        });

        let address = server_handle
            .listening()
            .await
            .context("HTTPS Server failed to bind address")?;

        log::info!("HTTPS Server started on {}", address);

        Ok(HttpServer {
            server_handle,
            join_handle,
            address,
            secure: true,
        })
    }

    #[must_use]
    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    fn get_protocol(&self) -> &str {
        if self.secure {
            "https"
        } else {
            "http"
        }
    }

    pub fn get_url(&self) -> Result<reqwest::Url> {
        reqwest::Url::from_str(&format!("{}://{}/", self.get_protocol(), self.address))
            .context("failed to create url from server address")
    }

    pub async fn shutdown(self) -> Result<()> {
        self.server_handle
            .graceful_shutdown(Some(Duration::from_secs(5)));

        self.join_handle.await.context("failed to join")??;

        Ok(())
    }
}

pub struct AllowWhiteListedClients {
    clients: Mutex<HashSet<Certificate>>,
}

impl AllowWhiteListedClients {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Default::default(),
        })
    }

    pub fn add_client(&self, certificate: Vec<u8>) -> Result<()> {
        let certificate = Certificate(certificate);

        self.clients
            .lock()
            .map_err(|err| anyhow!("Failed to lock clients map: {err}"))?
            .insert(certificate);

        Ok(())
    }

    fn is_familiar_certificate(&self, certificate: &Certificate) -> Result<bool> {
        let is_familiar = self
            .clients
            .lock()
            .map_err(|err| anyhow!("Failed to lock clients map: {err}"))?
            .contains(certificate);

        Ok(is_familiar)
    }
}

impl ClientCertVerifier for AllowWhiteListedClients {
    fn client_auth_root_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        _now: std::time::SystemTime,
    ) -> std::prelude::v1::Result<rustls::server::ClientCertVerified, rustls::Error> {
        if !intermediates.is_empty() {
            return Err(rustls::Error::General(
                "intermidiate certificates not supported".to_string(),
            ));
        }

        let is_familiar = self
            .is_familiar_certificate(end_entity)
            .map_err(|err| rustls::Error::General(err.to_string()))?;

        if is_familiar {
            return Ok(rustls::server::ClientCertVerified::assertion());
        }

        Err(rustls::Error::InvalidCertificate(
            rustls::CertificateError::ApplicationVerificationFailure,
        ))
    }
}
