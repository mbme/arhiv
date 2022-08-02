use anyhow::{anyhow, bail, ensure, Context, Result};
use reqwest::{Client, StatusCode};

use rs_utils::{create_body_from_file, log, Download};

use crate::entities::*;

pub struct PrimeServerRPC {
    prime_url: String,
    downloads_dir: String,
}

impl PrimeServerRPC {
    pub fn new(prime_url: impl Into<String>, downloads_dir: impl Into<String>) -> Result<Self> {
        let prime_url = prime_url.into();
        let downloads_dir = downloads_dir.into();

        ensure!(!prime_url.is_empty(), "prime_url must not  be empty");

        Ok(PrimeServerRPC {
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

    pub async fn upload_blob(&self, blob: &BLOB) -> Result<()> {
        log::debug!("uploading BLOB {}", &blob.id);

        let body = create_body_from_file(&blob.file_path, 0, None).await?;

        let url = self.get_blob_url(&blob.id);

        let response = Client::new().post(&url).body(body).send().await?;

        match response.status() {
            StatusCode::OK => {
                log::info!("uploaded BLOB {}", &blob.id);
                Ok(())
            }
            StatusCode::CONFLICT => {
                log::info!("skipped uploading BLOB {}: already exists", &blob.id);
                Ok(())
            }
            _ => {
                log::error!("failed to upload BLOB {}: {:?}", &blob.id, response);

                Err(anyhow!("failed to upload BLOB"))
            }
        }
    }

    pub async fn send_changeset(&self, changeset: &Changeset) -> Result<ChangesetResponse> {
        log::debug!("sending changeset...");

        let response = Client::new()
            .post(&format!("{}/changeset", self.prime_url))
            .json(&changeset)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            body.parse().context("failed to parse server response")
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
