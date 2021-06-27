use anyhow::*;
use futures::stream::TryStreamExt;
use reqwest::{Client, StatusCode};
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{arhiv::AttachmentData, entities::*};
use rs_utils::{
    log::{debug, error, info},
    read_file_as_stream,
};

pub struct NetworkService {
    prime_url: String,
}

impl NetworkService {
    pub fn new<S: Into<String>>(prime_url: S) -> Self {
        NetworkService {
            prime_url: prime_url.into(),
        }
    }

    pub async fn download_attachment_data(&self, attachment_data: &AttachmentData) -> Result<()> {
        if attachment_data.exists()? {
            bail!(
                "can't download attachment data: file {} already exists",
                attachment_data.path
            );
        }

        debug!(
            "downloading attachment data {} into {}",
            &attachment_data.id, &attachment_data.path
        );

        let url = self.get_attachment_data_url(&attachment_data.id);

        let mut stream = reqwest::get(&url)
            .await?
            .error_for_status()?
            .bytes_stream()
            // Convert the stream into an futures::io::AsyncRead.
            // We must first convert the reqwest::Error into an futures::io::Error.
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        let mut file = tokio_fs::File::create(&attachment_data.path).await?;

        // Invoke tokio::io::copy to actually perform the download.
        tokio::io::copy(&mut stream, &mut file).await?;

        Ok(())
    }

    pub async fn upload_attachment_data(&self, attachment_data: &AttachmentData) -> Result<()> {
        debug!("uploading attachment {}", &attachment_data.id);

        let file_stream = read_file_as_stream(&attachment_data.path).await?;

        let url = self.get_attachment_data_url(&attachment_data.id);

        let response = Client::new()
            .post(&url)
            .body(reqwest::Body::wrap_stream(file_stream))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                info!("uploaded attachment data {}", &attachment_data.id);
                Ok(())
            }
            StatusCode::CONFLICT => {
                info!(
                    "skipped uploading attachment data {}: already exists",
                    &attachment_data.id
                );
                Ok(())
            }
            _ => {
                error!(
                    "failed to upload attachment data {}: {:?}",
                    &attachment_data.id, response
                );

                Err(anyhow!("failed to upload attachment data"))
            }
        }
    }

    pub async fn send_changeset(&self, changeset: &Changeset) -> Result<ChangesetResponse> {
        debug!("sending changeset...");

        let response: ChangesetResponse = Client::new()
            .post(&format!("{}/changeset", self.prime_url))
            .json(&changeset)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .parse()?;

        Ok(response)
    }

    pub fn get_attachment_data_url(&self, id: &Id) -> String {
        format!("{}/attachment-data/{}", self.prime_url, id)
    }
}
