use super::path_manager::PathManager;
use super::Id;
use anyhow::*;
use futures::stream::TryStreamExt;
use rs_utils::file_exists;
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub struct AttachmentData<'a> {
    pub id: Id,
    path_manager: &'a PathManager,
}

impl<'a> AttachmentData<'a> {
    pub fn new(id: Id, path_manager: &'a PathManager) -> AttachmentData<'a> {
        AttachmentData { id, path_manager }
    }

    pub fn get_committed_file_path(&self) -> String {
        self.path_manager.get_committed_file_path(&self.id)
    }

    pub fn get_staged_file_path(&self) -> String {
        self.path_manager.get_staged_file_path(&self.id)
    }

    pub fn committed_file_exists(&self) -> Result<bool> {
        file_exists(&self.get_committed_file_path())
    }

    pub fn staged_file_exists(&self) -> Result<bool> {
        file_exists(&self.get_staged_file_path())
    }

    pub fn get_url(&self) -> Result<String> {
        self.path_manager.get_attachment_data_url(&self.id)
    }

    pub async fn download_data(&self) -> Result<()> {
        let path = self.get_committed_file_path();
        if file_exists(&path)? {
            bail!(
                "can't download attachment data: file {} already exists",
                path
            );
        }

        log::debug!(
            "downloading attachment data for {} into {}",
            &self.id,
            &path
        );

        let mut stream = reqwest::get(&self.get_url()?)
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
}
