use crate::{entities::*, storage::AttachmentData};
use anyhow::*;
use futures::stream::TryStreamExt;
use reqwest::Client;
use rs_utils::{file_exists, read_file_as_stream};
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

    pub async fn download_attachment_data<'a>(
        &self,
        attachment_data: &AttachmentData<'a>,
    ) -> Result<()> {
        let path = attachment_data.get_committed_file_path();
        if file_exists(&path)? {
            bail!(
                "can't download attachment data: file {} already exists",
                path
            );
        }

        log::debug!(
            "downloading attachment data for {} into {}",
            &attachment_data.id,
            &path
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

        let mut file = tokio_fs::File::create(path).await?;

        // Invoke tokio::io::copy to actually perform the download.
        tokio::io::copy(&mut stream, &mut file).await?;

        Ok(())
    }

    pub async fn upload_attachment_data<'a>(
        &self,
        attachment_data: &AttachmentData<'a>,
    ) -> Result<()> {
        let file_path = attachment_data.get_staged_file_path();

        log::debug!(
            "uploading attachment {} ({})",
            &attachment_data.id,
            &file_path
        );

        let file_stream = read_file_as_stream(&file_path).await?;

        let url = self.get_attachment_data_url(&attachment_data.id);

        Client::new()
            .post(&url)
            .body(reqwest::Body::wrap_stream(file_stream))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn send_changeset(&self, changeset: &Changeset) -> Result<ChangesetResponse> {
        log::debug!("sending changeset...");

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

    pub fn get_attachment_data_url(&self, attachment_id: &Id) -> String {
        format!("{}/attachment-data/{}", self.prime_url, &attachment_id)
    }
}
