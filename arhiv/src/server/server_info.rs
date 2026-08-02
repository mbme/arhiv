use anyhow::Result;
use serde::Serialize;

use super::SelfSignedCertificate;

use super::server_lock::ArhivServerLock;
use super::ui_server::{BROWSER_BOOTSTRAP_PATH, HEALTH_PATH, UI_BASE_PATH};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub ui_url: String,
    pub browser_url: String,
    pub health_url: String,
    pub certificate: Vec<u8>,
    pub auth_token: String,
}

impl ServerInfo {
    pub fn new(
        port: u16,
        certificate: &SelfSignedCertificate,
        auth_token: String,
        browser_bootstrap_token: String,
    ) -> Self {
        let ui_url = Self::get_ui_base_url(port);
        let health_url = Self::get_health_url(port);

        ServerInfo {
            browser_url: format!(
                "https://localhost:{port}{BROWSER_BOOTSTRAP_PATH}?token={browser_bootstrap_token}"
            ),
            ui_url,
            health_url,
            certificate: certificate.certificate_der.clone(),
            auth_token,
        }
    }

    pub fn get_server_port(state_dir: &str) -> Result<Option<u16>> {
        let mut lock = ArhivServerLock::new(state_dir);

        // server isn't running
        if lock.acquire().is_ok() {
            return Ok(None);
        }

        let port = lock.read_server_info()?;

        if port == 0 {
            return Ok(None);
        }

        Ok(Some(port))
    }

    fn get_ui_base_url(port: u16) -> String {
        format!("https://localhost:{port}{UI_BASE_PATH}")
    }

    fn get_health_url(port: u16) -> String {
        format!("https://localhost:{port}{HEALTH_PATH}")
    }
}
