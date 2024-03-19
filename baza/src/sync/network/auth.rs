use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};

use rs_utils::{hex_string_to_bytes, http_server::TlsData, log, ResponseVerifier, HMAC};

use crate::Baza;

pub const CERTIFICATE_HMAC_HEADER: &str = "X-Certificate-HMAC-Tag";

pub fn create_shared_network_key(baza: &Baza) -> Result<HMAC> {
    let conn = baza.get_connection()?;

    let app_name = baza.get_app_name();
    let login = conn.get_login()?;
    let password = conn.get_password()?;

    let hmac = HMAC::new_from_password(&password, &format!("{login}@{app_name}"))?;

    Ok(hmac)
}

#[derive(Clone)]
pub struct AuthInfo {
    pub hmac: Arc<HMAC>,
    pub server_cert_hmac_tag: String,
}

pub async fn client_cert_validator(
    Extension(AuthInfo {
        hmac,
        server_cert_hmac_tag,
    }): Extension<AuthInfo>,
    tls_data: Extension<TlsData>,
    request: Request,
    next: Next,
) -> Response {
    if tls_data.certificates.len() != 1 {
        return with_hmac_tag_header(
            (
                StatusCode::BAD_REQUEST,
                format!(
                    "Expected 1 TLS certificate, got {}",
                    tls_data.certificates.len()
                ),
            ),
            server_cert_hmac_tag,
        );
    }

    let client_cert = tls_data
        .certificates
        .first()
        .expect("certificate must be present");

    let client_cert_hmac = request
        .headers()
        .get(CERTIFICATE_HMAC_HEADER)
        .context(format!("{CERTIFICATE_HMAC_HEADER} header is missing"))
        .and_then(|value| {
            value.to_str().context(format!(
                "Failed to read {CERTIFICATE_HMAC_HEADER} header as string"
            ))
        })
        .and_then(hex_string_to_bytes);

    let client_cert_hmac = match client_cert_hmac {
        Ok(client_cert_hmac) => client_cert_hmac,
        Err(err) => {
            return with_hmac_tag_header(
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read {CERTIFICATE_HMAC_HEADER} header: {err}"),
                ),
                server_cert_hmac_tag,
            )
        }
    };

    if !hmac.verify(client_cert, &client_cert_hmac) {
        log::warn!("Got unknown client certificate");

        return with_hmac_tag_header(
            (
                StatusCode::UNAUTHORIZED,
                "Invalid HMAC tag for client TLS certificate",
            ),
            server_cert_hmac_tag,
        );
    }

    let response = next.run(request).await;

    with_hmac_tag_header(response, server_cert_hmac_tag)
}

fn with_hmac_tag_header(response: impl IntoResponse, hmac_tag: String) -> Response {
    let mut response = response.into_response();

    response.headers_mut().append(
        CERTIFICATE_HMAC_HEADER,
        hmac_tag
            .parse()
            .expect("Failed to convert server certificate HMAC tag into header value"),
    );

    response
}

#[derive(Debug)]
pub struct ServerCertVerifier {
    hmac: HMAC,
}

impl ServerCertVerifier {
    pub fn new(hmac: HMAC) -> Self {
        ServerCertVerifier { hmac }
    }
}

impl ResponseVerifier for ServerCertVerifier {
    fn verify(&self, response: &reqwest::Response) -> Result<()> {
        let server_cert = response
            .extensions()
            .get::<reqwest::tls::TlsInfo>()
            .context("TlsInfo is missing")?
            .peer_certificate()
            .context("Server certificate is missing")?;

        let server_cert_hmac = response
            .headers()
            .get(CERTIFICATE_HMAC_HEADER)
            .context(format!("{CERTIFICATE_HMAC_HEADER} header is missing"))?
            .to_str()
            .context(format!(
                "Failed to read {CERTIFICATE_HMAC_HEADER} header as string"
            ))?;
        let server_cert_hmac = hex_string_to_bytes(server_cert_hmac)?;

        ensure!(
            self.hmac.verify(server_cert, &server_cert_hmac),
            "Got invalid HMAC tag for server certificate"
        );

        Ok(())
    }
}
