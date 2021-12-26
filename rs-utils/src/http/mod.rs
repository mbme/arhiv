use anyhow::{anyhow, ensure, Result};
use futures::stream::TryStreamExt;
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use url::Url;

use crate::{file_exists, get_downloads_dir};

pub mod server;

pub async fn download_data_to_file(url: &str, file_path: &str) -> Result<()> {
    let mut stream = reqwest::get(url)
        .await?
        .error_for_status()?
        .bytes_stream()
        // Convert the stream into an futures::io::AsyncRead.
        // We must first convert the reqwest::Error into an futures::io::Error.
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = tokio_fs::File::create(file_path).await?;

    // Invoke tokio::io::copy to actually perform the download.
    tokio::io::copy(&mut stream, &mut file).await?;

    Ok(())
}

pub fn extract_file_name_from_url(url: &str) -> Result<Option<String>> {
    let url = Url::parse(url)?;

    let file_name = url
        .path_segments()
        .and_then(Iterator::last)
        .map(ToString::to_string);

    Ok(file_name)
}

pub async fn download_file(src_url: &str) -> Result<String> {
    let downloads_dir =
        get_downloads_dir().ok_or_else(|| anyhow!("failed to find Downloads dir"))?;

    // TODO add ".download" suffix while downloading
    // TOOD support (pausing/)restoring download, support HTTP Ranges

    // TODO also read filename from the Content-Disposition header
    // Content-Disposition: attachment; filename="filename.jpg"
    let file_name = extract_file_name_from_url(src_url)?
        .ok_or_else(|| anyhow!("failed to extract file name from url {}", src_url))?;

    let file = format!("{}/{}", &downloads_dir, file_name);
    ensure!(!file_exists(&file)?, "file {} already exists", file);

    download_data_to_file(src_url, &file).await?;
    log::debug!("Downloaded {} to {}", src_url, &file);

    Ok(file)
}
