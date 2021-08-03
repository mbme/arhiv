use anyhow::*;
use async_trait::async_trait;
use url::Url;

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::{download_data_to_file, file_exists, get_downloads_dir, log, run_js_script};

#[async_trait]
pub trait Importer {
    fn can_import(&self, url: &str) -> bool;

    async fn import(&self, data: &str, arhiv: &Arhiv) -> Result<Document>;
}

pub fn scrape(url: &str) -> Result<String> {
    let script = include_str!("../dist/bundle.js");

    run_js_script(script, vec![url])
}

pub async fn download_file(src_url: &str) -> Result<String> {
    let downloads_dir = get_downloads_dir().ok_or(anyhow!("failed to find Downloads dir"))?;

    // extract file name from url
    let url = Url::parse(src_url)?;
    let file_name = url
        .path_segments()
        .map(|segments| segments.last())
        .flatten()
        .ok_or(anyhow!("failed to extract file name from url {}", src_url))?;

    let file = format!("{}/{}", &downloads_dir, file_name);
    ensure!(!file_exists(&file)?, "file {} already exists", file);

    download_data_to_file(src_url, &file).await?;
    log::debug!("Downloaded {} to {}", src_url, &file);

    Ok(file)
}
