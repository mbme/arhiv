use anyhow::{anyhow, bail, ensure, Context, Result};
use reqwest::Client;

use baza::{
    entities::*,
    sync::{changeset::Changeset, Revision},
};
use rs_utils::{log, Download};

pub struct BazaRpcClient {
    prime_url: String,
    downloads_dir: String,
}

impl BazaRpcClient {
    pub fn new(prime_url: impl Into<String>, downloads_dir: impl Into<String>) -> Result<Self> {
        let prime_url = prime_url.into();
        let downloads_dir = downloads_dir.into();

        ensure!(!prime_url.is_empty(), "prime_url must not  be empty");

        Ok(BazaRpcClient {
            prime_url,
            downloads_dir,
        })
    }

    pub async fn download_blob(&self, blob: &BLOB) -> Result<()> {
        if blob.exists()? {
            bail!(
                "can't download BLOB: file {} already exists",
                blob.file_path
            );
        }

        log::debug!("downloading BLOB {}", &blob.id);

        let url = self.get_blob_url(&blob.id);

        let mut download = Download::new_with_path(&url, &self.downloads_dir)?;
        download.keep_completed_file(true);
        download.keep_download_file(true);

        let download_result = download.start().await?;

        tokio::fs::rename(&download_result.file_path, &blob.file_path)
            .await
            .context("failed to move downloaded blob into blob dir")?;

        log::debug!("downloaded BLOB {}", &blob.id);

        Ok(())
    }

    pub async fn get_changeset(&self, min_rev: Revision) -> Result<Changeset> {
        let min_rev = min_rev.serialize();
        log::debug!("fetching a changeset since {}", min_rev);

        let response = Client::new()
            .get(&format!("{}/changeset/{}", self.prime_url, min_rev))
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).context("failed to parse server response")
        } else {
            Err(anyhow!("Server responded with error: {}\n{}", status, body))
        }
    }

    fn get_blob_url(&self, blob_id: &BLOBId) -> String {
        format!("{}/blobs/{}", self.prime_url, blob_id)
    }

    pub async fn check_connection(&self) -> Result<()> {
        Client::new()
            .get(&format!("{}/status", self.prime_url))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
