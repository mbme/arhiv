use anyhow::Result;
use serde::Serialize;

use baza::entities::Id;
use rs_utils::{AuthToken, SelfSignedCertificate, Token};

use super::certificate::{generate_ui_crypto_key, read_or_generate_certificate};
use super::server_lock::ArhivServerLock;
use super::ui_server::{HEALTH_PATH, UI_BASE_PATH};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub ui_url: String,
    pub ui_url_with_auth_token: String,
    pub health_url: String,
    pub certificate: Vec<u8>,
    pub auth_token: String,
}

impl ServerInfo {
    pub fn new(port: u16, certificate: &SelfSignedCertificate, token: Token) -> Self {
        let ui_url = Self::get_ui_base_url(port);
        let health_url = Self::get_health_url(port);

        let ui_hmac = generate_ui_crypto_key(certificate.private_key_der.clone());
        let auth_token = AuthToken::generate(&ui_hmac, token).serialize();

        ServerInfo {
            ui_url_with_auth_token: format!("{ui_url}?AuthToken={auth_token}"),
            ui_url,
            health_url,
            certificate: certificate.certificate_der.clone(),
            auth_token,
        }
    }

    pub fn collect(root_dir: &str) -> Result<Option<Self>> {
        let (port, token) = if let Some((port, token)) = Self::get_server_info(root_dir)? {
            (port, token)
        } else {
            return Ok(None);
        };

        let certificate = read_or_generate_certificate(root_dir)?;
        let info = ServerInfo::new(port, &certificate, token);

        Ok(Some(info))
    }

    pub fn get_server_info(root_dir: &str) -> Result<Option<(u16, Token)>> {
        let mut lock = ArhivServerLock::new(root_dir);

        // server isn't running
        if lock.acquire().is_ok() {
            return Ok(None);
        }

        let (port, token) = lock.read_server_info()?;

        if port == 0 {
            return Ok(None);
        }

        Ok(Some((port, token)))
    }

    fn get_ui_base_url(port: u16) -> String {
        format!("https://localhost:{port}{UI_BASE_PATH}")
    }

    fn get_health_url(port: u16) -> String {
        format!("https://localhost:{port}{HEALTH_PATH}")
    }

    pub fn get_document_url(&self, id: &Id) -> String {
        format!("{}&id={id}", self.ui_url_with_auth_token)
    }
}
