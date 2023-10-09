use anyhow::{bail, Context, Result};
use reqwest::{header, Client, Url};

use rs_utils::{log, Download};

use crate::{
    entities::BLOB,
    sync::{changeset::Changeset, ping::Ping, Revision},
};

#[derive(Debug)]
pub struct BazaClient {
    rpc_server_url: Url,
    downloads_dir: String,
}

// TODO support timeouts
impl BazaClient {
    pub fn new(rpc_server_url: Url, downloads_dir: impl Into<String>) -> Self {
        let downloads_dir = downloads_dir.into();

        BazaClient {
            rpc_server_url,
            downloads_dir,
        }
    }

    pub async fn download_blob(&self, blob: &BLOB) -> Result<()> {
        log::debug!(
            "Baza Server {}: downloading BLOB {}",
            self.rpc_server_url,
            blob.id
        );

        if blob.exists()? {
            bail!(
                "can't download BLOB: file {} already exists",
                blob.file_path
            );
        }

        let blob_url = self.rpc_server_url.join("/blobs/")?.join(&blob.id)?;

        let mut download = Download::new_with_path(blob_url.as_str(), &self.downloads_dir)?;
        download.keep_completed_file(true);
        download.keep_download_file(true);

        let download_result = download.start().await?;

        tokio::fs::rename(&download_result.file_path, &blob.file_path)
            .await
            .context("failed to move downloaded blob into blob dir")?;

        log::debug!(
            "Baza Server {}: downloaded BLOB {}",
            self.rpc_server_url,
            blob.id
        );

        Ok(())
    }

    pub async fn exchange_pings(&self, ping: &Ping) -> Result<Ping> {
        log::debug!("Baza Server {}: exchanging pings", self.rpc_server_url);

        let body = serde_json::to_vec(ping).context("failed to serialize ping")?;
        let response = Client::new()
            .post(self.rpc_server_url.join("/ping")?)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).context("failed to parse server response")
        } else {
            bail!("Server responded with error: {}\n{}", status, body);
        }
    }

    pub async fn get_changeset(&self, min_rev: &Revision) -> Result<Changeset> {
        let min_rev = min_rev.serialize();
        log::debug!(
            "Baza Server {}: fetching a changeset since {min_rev}",
            self.rpc_server_url,
        );

        let response = Client::new()
            .get(self.rpc_server_url.join("/changeset/")?.join(&min_rev)?)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).context("failed to parse server response")
        } else {
            bail!("Server responded with error: {}\n{}", status, body);
        }
    }

    pub async fn check_connection(&self) -> Result<()> {
        Client::new()
            .get(self.rpc_server_url.join("/status")?)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
