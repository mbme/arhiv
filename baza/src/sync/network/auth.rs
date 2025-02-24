use std::sync::Arc;

use anyhow::{ensure, Context, Result};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};

use rs_utils::{
    bytes_to_hex_string, crypto_key::CryptoKey, hex_string_to_bytes,
    http_server::ServerCertificate, log, AuthToken, ResponseVerifier,
};

use crate::Baza;

pub const CERTIFICATE_SIGNATURE_HEADER: &str = "X-Certificate-Signature";
pub const CLIENT_AUTH_TOKEN_HEADER: &str = "X-Client-Auth-Token";

pub async fn client_authenticator(
    Extension(key): Extension<Arc<CryptoKey>>,
    Extension(server_cert): Extension<ServerCertificate>,
    request: Request,
    next: Next,
) -> Response {
    let server_cert_signature = bytes_to_hex_string(&key.sign(server_cert.as_ref()));

    let client_auth_token = request
        .headers()
        .get(CLIENT_AUTH_TOKEN_HEADER)
        .context(format!("{CLIENT_AUTH_TOKEN_HEADER} header is missing"))
        .and_then(|value| {
            value.to_str().context(format!(
                "Failed to read {CLIENT_AUTH_TOKEN_HEADER} header as string"
            ))
        })
        .and_then(AuthToken::parse);

    let client_auth_token = match client_auth_token {
        Ok(client_auth_token) => client_auth_token,
        Err(err) => {
            return with_signature_header(
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read {CLIENT_AUTH_TOKEN_HEADER} header: {err}"),
                ),
                server_cert_signature,
            )
        }
    };

    if let Err(err) = client_auth_token.assert_is_valid(key.as_ref()) {
        log::warn!("Got unauthenticated client: {err}");

        return with_signature_header(
            (StatusCode::UNAUTHORIZED, "Invalid client auth token"),
            server_cert_signature,
        );
    }

    let response = next.run(request).await;

    with_signature_header(response, server_cert_signature)
}

fn with_signature_header(response: impl IntoResponse, signature: String) -> Response {
    let mut response = response.into_response();

    response.headers_mut().append(
        CERTIFICATE_SIGNATURE_HEADER,
        signature
            .parse()
            .expect("Failed to convert server certificate signature into header value"),
    );

    response
}

pub struct ServerCertVerifier {
    key: CryptoKey,
}

impl ServerCertVerifier {
    pub fn new(key: CryptoKey) -> Self {
        ServerCertVerifier { key }
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

        let server_cert_signature = response
            .headers()
            .get(CERTIFICATE_SIGNATURE_HEADER)
            .context(format!("{CERTIFICATE_SIGNATURE_HEADER} header is missing"))?
            .to_str()
            .context(format!(
                "Failed to read {CERTIFICATE_SIGNATURE_HEADER} header as string"
            ))?;
        let server_cert_signature = hex_string_to_bytes(server_cert_signature)?;

        ensure!(
            self.key
                .verify_signature(server_cert, &server_cert_signature),
            "Got invalid signature for server certificate"
        );

        Ok(())
    }
}

pub fn create_shared_key(baza: &Baza) -> Result<CryptoKey> {
    let conn = baza.get_connection()?;

    let app_name = baza.get_app_name();
    let login = conn.get_login()?;
    let password = conn.get_password()?;

    CryptoKey::derive_from_password_with_scrypt(
        &password,
        CryptoKey::salt_from_data(format!("{login}@{app_name}"))?,
    )
}
