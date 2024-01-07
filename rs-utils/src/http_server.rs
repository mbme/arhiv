use std::{
    net::SocketAddr,
    str::FromStr,
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
use reqwest::Client;
use tokio::{net::TcpListener, sync::broadcast, task::JoinHandle};

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
    let url = reqwest::Url::from_str(&format!("http://{server_url}/health"))
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
    shutdown_sender: broadcast::Sender<()>,
    join_handle: JoinHandle<Result<()>>,
}

impl HttpServer {
    pub async fn start(router: Router, port: u16) -> Result<HttpServer> {
        let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(1);

        let listener = TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, port))
            .await
            .context("failed to bind TCP listener")?;
        let address = listener.local_addr()?;

        // Spawn the server into a runtime
        let join_handle = tokio::spawn(async {
            let router = router.layer(middleware::from_fn(logger_middleware));

            let server = axum::serve(listener, router.into_make_service());

            server
                .with_graceful_shutdown(async move {
                    if let Err(err) = shutdown_receiver.recv().await {
                        log::error!("HTTP Server: failed to get shutdown signal: {err}");
                    } else {
                        log::info!("HTTP Server: got shutdown signal");
                    }
                })
                .await
                .context("HTTP Server failed to start")?;

            log::info!("HTTP Server exited");

            Ok(())
        });

        log::info!("HTTP Server: started on {}", address);

        Ok(HttpServer {
            join_handle,
            shutdown_sender,
            address,
        })
    }

    #[must_use]
    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn get_url(&self) -> Result<reqwest::Url> {
        reqwest::Url::from_str(&format!("http://{}/", self.address))
            .context("failed to create url from server address")
    }

    pub async fn shutdown(self) -> Result<()> {
        self.shutdown_sender
            .send(())
            .map_err(|_err| anyhow!("receiver dropped"))?;

        self.join_handle.await.context("failed to join")??;

        Ok(())
    }
}
