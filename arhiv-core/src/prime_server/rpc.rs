use anyhow::{anyhow, bail, ensure, Context, Result};
use reqwest::{Client, StatusCode};

use rs_utils::{log, read_file_as_stream, Download};

use crate::entities::*;

pub struct PrimeServerRPC {
    prime_url: String,
}

impl PrimeServerRPC {
    pub fn new<S: Into<String>>(prime_url: S) -> Result<Self> {
        let prime_url = prime_url.into();

        ensure!(!prime_url.is_empty(), "prime_url must not  be empty");

        Ok(PrimeServerRPC { prime_url })
    }

    pub async fn download_blob(&self, blob: &BLOB) -> Result<()> {
        if blob.exists()? {
            bail!(
                "can't download BLOB: file {} already exists",
                blob.file_path
            );
        }

        log::debug!("downloading BLOB {} into {}", &blob.id, &blob.file_path);

        let url = self.get_blob_url(&blob.id);

        let mut download = Download::new_with_path(&url, &blob.file_path)?;
        download.keep_completed_file(true);
        download.keep_download_file(true);

        download.start().await?;

        Ok(())
    }

    pub async fn upload_blob(&self, blob: &BLOB) -> Result<()> {
        log::debug!("uploading BLOB {}", &blob.id);

        let file_stream = read_file_as_stream(&blob.file_path).await?;

        let url = self.get_blob_url(&blob.id);

        let response = Client::new()
            .post(&url)
            .body(reqwest::Body::wrap_stream(file_stream))
            .send()
            .await?;

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
}
