use anyhow::*;
use reqwest::{Client, StatusCode};

use crate::{arhiv::AttachmentData, entities::*};
use rs_utils::{download_data_to_file, log, read_file_as_stream};

pub struct PrimeServerRPC {
    prime_url: String,
}

impl PrimeServerRPC {
    pub fn new<S: Into<String>>(prime_url: S) -> Result<Self> {
        let prime_url = prime_url.into();

        ensure!(!prime_url.is_empty(), "prime_url must not  be empty");

        Ok(PrimeServerRPC { prime_url })
    }

    pub async fn download_attachment_data(&self, attachment_data: &AttachmentData) -> Result<()> {
        if attachment_data.exists()? {
            bail!(
                "can't download attachment data: file {} already exists",
                attachment_data.path
            );
        }

        log::debug!(
            "downloading attachment data {} into {}",
            &attachment_data.id,
            &attachment_data.path
        );

        let url = self.get_attachment_data_url(&attachment_data.id);

        download_data_to_file(&url, &attachment_data.path).await?;

        Ok(())
    }

    pub async fn upload_attachment_data(&self, attachment_data: &AttachmentData) -> Result<()> {
        log::debug!("uploading attachment {}", &attachment_data.id);

        let file_stream = read_file_as_stream(&attachment_data.path).await?;

        let url = self.get_attachment_data_url(&attachment_data.id);

        let response = Client::new()
            .post(&url)
            .body(reqwest::Body::wrap_stream(file_stream))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                log::info!("uploaded attachment data {}", &attachment_data.id);
                Ok(())
            }
            StatusCode::CONFLICT => {
                log::info!(
                    "skipped uploading attachment data {}: already exists",
                    &attachment_data.id
                );
                Ok(())
            }
            _ => {
                log::error!(
                    "failed to upload attachment data {}: {:?}",
                    &attachment_data.id,
                    response
                );

                Err(anyhow!("failed to upload attachment data"))
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

    fn get_attachment_data_url(&self, id: &Id) -> String {
        format!("{}/attachment-data/{}", self.prime_url, id)
    }
}
