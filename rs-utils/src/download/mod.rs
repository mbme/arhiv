mod file_name_expert;

use std::time::Duration;

use anyhow::{bail, ensure, Context, Result};
use futures::stream::TryStreamExt;
use reqwest::{Client, Response};
use tokio::fs as tokio_fs;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use url::Url;

use crate::{
    download::file_name_expert::DownloadFileNameExpert,
    ensure_dir_exists, file_exists, get_downloads_dir, get_file_size, get_string_hash_blake3,
    http::{
        parse_content_disposition_header, parse_content_range_header, parse_content_type_header,
    },
    log, remove_file_if_exists,
};

pub struct DownloadResult {
    pub original_file_name: String,
    pub file_path: String,
    keep_completed_file: bool,
}

const DOWNLOAD_SUFFIX: &str = ".wip-download";

pub struct Download {
    url: Url,
    completed_file_path: String,
    download_file_path: String,
    keep_download_file: bool,
    keep_completed_file: bool,
    client: Client,
}

impl Download {
    pub fn new(url: &str) -> Result<Self> {
        let downloads_dir = get_downloads_dir().context("failed to find Downloads dir")?;

        Download::new_with_path(url, &downloads_dir)
    }

    pub fn new_with_path(url: &str, downloads_dir: &str) -> Result<Self> {
        ensure_dir_exists(downloads_dir).context("dir for downloads doesn't exist")?;

        let url_hash = get_string_hash_blake3(url);
        let completed_file_path = format!("{downloads_dir}/{url_hash}");

        // FIXME better check if download is complete
        ensure!(
            !file_exists(&completed_file_path)?,
            "Completed download file {} already exists",
            completed_file_path
        );

        let download_file_path = format!("{}{}", &completed_file_path, DOWNLOAD_SUFFIX);

        let url = Url::parse(url).context("failed to parse url")?;

        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .build()
            .context("failed to build client")?;

        Ok(Download {
            url,
            completed_file_path,
            download_file_path,
            keep_download_file: false,
            keep_completed_file: false,
            client,
        })
    }

    pub fn keep_download_file(&mut self, keep: bool) {
        self.keep_download_file = keep;
    }

    pub fn keep_completed_file(&mut self, keep: bool) {
        self.keep_completed_file = keep;
    }

    pub fn use_custom_http_client(&mut self, client: Client) {
        self.client = client;
    }

    fn deduce_start_pos(&self) -> Result<u64> {
        let start_pos = if file_exists(&self.download_file_path)? {
            log::debug!("Download file {} already exists", self.download_file_path);

            get_file_size(&self.download_file_path)?
        } else {
            0
        };

        Ok(start_pos)
    }

    async fn send_request(&self, start_pos: u64) -> Result<Response> {
        let mut request = self.client.get(self.url.clone());

        if start_pos > 0 {
            request = request.header(reqwest::header::RANGE, format!("bytes={start_pos}-"));
        }

        request.send().await.context("failed to send request")
    }

    async fn open_download_file(&self, start_pos: u64) -> Result<tokio_fs::File> {
        let mut options = tokio_fs::OpenOptions::new();
        options.write(true).create(true);

        if start_pos == 0 {
            options.truncate(true);
        } else {
            options.append(true);
        }

        options
            .open(&self.download_file_path)
            .await
            .context("failed to open download file for write")
    }

    pub async fn start(&self) -> Result<DownloadResult> {
        log::info!(
            "Starting download of {} into {}",
            &self.url,
            &self.download_file_path
        );

        let mut start_pos = self.deduce_start_pos()?;

        let response = self.send_request(start_pos).await?;

        // FIXME validate response

        let status = response.status();
        log::debug!("HTTP response status: {}", &status);

        match status {
            reqwest::StatusCode::OK => {
                if start_pos > 0 {
                    log::warn!("Sent Partial HTTP request but server returned regular response");

                    start_pos = 0;
                }
            }

            reqwest::StatusCode::PARTIAL_CONTENT => {
                if start_pos == 0 {
                    log::warn!("Sent regular HTTP request but server returned Partial response");
                }
            }
            _ => {
                bail!("Download failed: unexpected response status: {}", &status);
            }
        }

        let mut expected_file_size = response.content_length();

        if let Some(content_range) = response.headers().get(reqwest::header::CONTENT_RANGE) {
            let content_range = content_range.to_str()?;

            let (start, end, size) = parse_content_range_header(content_range)?;

            ensure!(
                start == start_pos,
                "Content-Range start pos {} must be equal to expected start pos {}",
                start,
                start_pos
            );

            ensure!(
                end == size - 1,
                "Content-Range end pos {} is smaller than expected end pos {}",
                end,
                size - 1
            );

            expected_file_size = Some(size);
        }

        let content_type =
            if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
                let content_type = content_type.to_str()?;

                Some(parse_content_type_header(content_type)?.0)
            } else {
                None
            };

        let attachment_file_name = if let Some(content_disposition) =
            response.headers().get(reqwest::header::CONTENT_DISPOSITION)
        {
            let content_disposition = content_disposition.to_str()?;

            parse_content_disposition_header(content_disposition)?
        } else {
            None
        };

        let mut stream = response
            .bytes_stream()
            // Convert the stream into an futures::io::AsyncRead.
            // We must first convert the reqwest::Error into an futures::io::Error.
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        let mut file = self.open_download_file(start_pos).await?;

        // Invoke tokio::io::copy to actually perform the download.
        tokio::io::copy(&mut stream, &mut file).await?;

        if let Some(expected_file_size) = expected_file_size {
            let file_size = get_file_size(&self.download_file_path)?;

            ensure!(
                file_size == expected_file_size,
                "File size {} doesn't match expected file size {}",
                file_size,
                expected_file_size
            );
        } else {
            log::warn!("Coudn't deduce expected file size, file size check skipped");
        }

        tokio_fs::rename(&self.download_file_path, &self.completed_file_path)
            .await
            .context("failed to rename download file")?;

        let original_file_name = DownloadFileNameExpert {
            url: &self.url,
            attachment_file_name: attachment_file_name.unwrap_or_default(),
            content_type,
            file_path: self.completed_file_path.clone(),
        }
        .deduce_file_name()
        .context("failed to deduce original file name")?;

        log::debug!("Deduced file name: {}", original_file_name);

        Ok(DownloadResult {
            original_file_name,
            file_path: self.completed_file_path.clone(),
            keep_completed_file: self.keep_completed_file,
        })
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        if self.keep_download_file {
            return;
        }

        if let Err(err) = remove_file_if_exists(&self.download_file_path) {
            log::warn!(
                "Failed to remove Download file {}: {}",
                self.download_file_path,
                err
            );
        }
    }
}

impl Drop for DownloadResult {
    fn drop(&mut self) {
        if self.keep_completed_file {
            return;
        }

        if let Err(err) = remove_file_if_exists(&self.file_path) {
            log::warn!(
                "Failed to remove Completed Download file {}: {}",
                self.file_path,
                err
            );
        }
    }
}
