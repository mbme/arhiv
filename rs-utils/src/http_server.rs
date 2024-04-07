use std::{
    io,
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, AddExtension, Next},
    response::{IntoResponse, Response},
    Extension, Router,
};
use axum_extra::headers::{self, HeaderMapExt};
use axum_server::{
    accept::Accept,
    tls_rustls::{RustlsAcceptor, RustlsConfig},
    Handle, Server,
};
use futures::future::BoxFuture;
use hyper::Uri;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    task::JoinHandle,
};
use tokio_rustls::{
    rustls::{
        server::{ClientCertVerified, ClientCertVerifier},
        Certificate, PrivateKey, ServerConfig,
    },
    server::TlsStream,
};
use tower::Layer;

use crate::SelfSignedCertificate;

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

async fn logger_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
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

pub async fn fallback_route(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
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
        server_certificate: SelfSignedCertificate,
    ) -> Result<Self> {
        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(RequireAnyClientCertificate))
            .with_single_cert(
                vec![Certificate(server_certificate.certificate_der)],
                PrivateKey(server_certificate.private_key_der.as_bytes().to_vec()),
            )?;
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        let config = RustlsConfig::from_config(Arc::new(config));

        let server_handle = Handle::new();

        let addr: SocketAddr = (std::net::Ipv4Addr::UNSPECIFIED, port).into();

        let acceptor = CustomAcceptor::new(RustlsAcceptor::new(config));
        let server = Server::bind(addr)
            .acceptor(acceptor)
            .handle(server_handle.clone());

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

#[derive(Debug, Clone)]
pub struct TlsData {
    pub certificates: Arc<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone)]
struct CustomAcceptor {
    inner: RustlsAcceptor,
}

impl CustomAcceptor {
    fn new(inner: RustlsAcceptor) -> Self {
        Self { inner }
    }
}

// based on https://github.com/programatik29/axum-server/blob/master/examples/rustls_session.rs
impl<I, S> Accept<I, S> for CustomAcceptor
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    S: Send + 'static,
{
    type Stream = TlsStream<I>;
    type Service = AddExtension<S, TlsData>;
    type Future = BoxFuture<'static, io::Result<(Self::Stream, Self::Service)>>;

    fn accept(&self, stream: I, service: S) -> Self::Future {
        let acceptor = self.inner.clone();

        Box::pin(async move {
            let (stream, service) = acceptor.accept(stream, service).await?;
            let server_conn = stream.get_ref().1;
            let certificates = server_conn
                .peer_certificates()
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|cert| cert.0)
                .collect();

            let tls_data = TlsData {
                certificates: Arc::new(certificates),
            };
            let service = Extension(tls_data).layer(service);

            Ok((stream, service))
        })
    }
}

struct RequireAnyClientCertificate;

impl ClientCertVerifier for RequireAnyClientCertificate {
    fn client_auth_root_subjects(&self) -> &[tokio_rustls::rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _now: std::time::SystemTime,
    ) -> std::prelude::v1::Result<
        tokio_rustls::rustls::server::ClientCertVerified,
        tokio_rustls::rustls::Error,
    > {
        Ok(ClientCertVerified::assertion())
    }
}
