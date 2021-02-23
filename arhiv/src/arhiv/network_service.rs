use crate::{arhiv::AttachmentData, entities::*};
use anyhow::*;
use futures::stream::TryStreamExt;
use reqwest::Client;
use rs_utils::{log::debug, read_file_as_stream};
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;

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
            &attachment_data.hash, &attachment_data.path
        );

        let url = self.get_attachment_data_url(&attachment_data.hash);

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
        debug!(
            "uploading attachment {} ({})",
            &attachment_data.hash, &attachment_data.path
        );

        let file_stream = read_file_as_stream(&attachment_data.path).await?;

        let url = self.get_attachment_data_url(&attachment_data.hash);

        Client::new()
            .post(&url)
            .body(reqwest::Body::wrap_stream(file_stream))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
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

    pub fn get_attachment_data_url(&self, hash: &Hash) -> String {
        format!("{}/attachment-data/{}", self.prime_url, hash)
    }
}
