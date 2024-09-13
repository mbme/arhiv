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
    bytes_to_hex_string, hex_string_to_bytes, http_server::ServerCertificate, log, AuthToken,
    ResponseVerifier, HMAC,
};

use crate::Baza;

pub const CERTIFICATE_HMAC_HEADER: &str = "X-Certificate-HMAC-Tag";
pub const CLIENT_AUTH_TOKEN_HEADER: &str = "X-Client-Auth-Token";

pub fn create_shared_network_verifier(baza: &Baza) -> Result<HMAC> {
    let conn = baza.get_connection()?;

    let app_name = baza.get_app_name();
    let login = conn.get_login()?;
    let password = conn.get_password()?;

    let hmac = HMAC::new_from_password(password, format!("{login}@{app_name}"))?;

    Ok(hmac)
}

pub async fn client_authenticator(
    Extension(baza): Extension<Arc<Baza>>,
    Extension(server_cert): Extension<ServerCertificate>,
    request: Request,
    next: Next,
) -> Response {
    // FIXME do not recreate the HMAC on each request
    let hmac = create_shared_network_verifier(&baza).unwrap();
    let server_cert_hmac_tag = bytes_to_hex_string(&hmac.sign(server_cert.as_ref()));

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
            return with_hmac_tag_header(
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read {CLIENT_AUTH_TOKEN_HEADER} header: {err}"),
                ),
                server_cert_hmac_tag,
            )
        }
    };

    if let Err(err) = client_auth_token.assert_is_valid(&hmac) {
        log::warn!("Got unauthenticated client: {err}");

        return with_hmac_tag_header(
            (StatusCode::UNAUTHORIZED, "Invalid client auth token"),
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
