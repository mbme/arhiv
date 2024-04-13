use anyhow::Result;
use serde::Serialize;

use baza::entities::Id;

use super::certificate::read_or_generate_certificate;
use super::server_lock::ArhivServerLock;
use super::ui_server::UI_BASE_PATH;

#[derive(Serialize)]
pub struct ServerInfo {
    url: String,
    certificate: Vec<u8>,
}

impl ServerInfo {
    pub fn collect(root_dir: &str) -> Result<Option<Self>> {
        let port = if let Some(port) = Self::get_server_port(root_dir)? {
            port
        } else {
            return Ok(None);
        };

        let url = Self::get_ui_base_url(port);

        let certificate = read_or_generate_certificate(root_dir)?.certificate_der;

        Ok(Some(Self { url, certificate }))
    }

    pub fn get_server_port(root_dir: &str) -> Result<Option<u16>> {
        let mut lock = ArhivServerLock::new(root_dir);

        // server isn't running
        if lock.acquire().is_ok() {
            return Ok(None);
        }

        let port = lock.read_server_port()?;

        if port == 0 {
            return Ok(None);
        }

        Ok(Some(port))
    }

    pub fn get_ui_base_url(port: u16) -> String {
        format!("https://localhost:{port}{UI_BASE_PATH}")
    }

    pub fn get_document_url(id: &Id, port: u16) -> String {
        let base = Self::get_ui_base_url(port);

        format!("{base}?id={id}")
    }
}
