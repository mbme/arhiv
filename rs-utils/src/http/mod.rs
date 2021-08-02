use anyhow::*;
use futures::stream::TryStreamExt;
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub use query_builder::QueryBuilder;

mod query_builder;
pub mod server;

pub fn get_mime_from_path(path: impl AsRef<str>) -> String {
    mime_guess::from_path(path.as_ref())
        .first_or_octet_stream()
        .to_string()
}

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
