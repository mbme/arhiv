use anyhow::*;
use async_trait::async_trait;
use dialoguer::{theme::ColorfulTheme, Confirm};
use url::Url;

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::{download_data_to_file, file_exists, get_downloads_dir, log, run_js_script};

#[async_trait]
pub trait Importer {
    fn get_name(&self) -> &str;

    fn can_import(&self, url: &str) -> bool;

    async fn import(&self, url: &str, arhiv: &Arhiv, confirm: bool) -> Result<Document>;
}

fn scrape(url: &str) -> Result<String> {
    let script = include_str!("../dist/bundle.js");

    run_js_script(script, vec![url])
}

pub fn confirm_if_needed(confirm: bool) -> Result<()> {
    if !confirm {
        return Ok(());
    }

    let proceed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you really want to continue?")
        .default(true)
        .interact()
        .context("failed to ask confirmation")?;

    if proceed {
        return Ok(());
    } else {
        bail!("confirmation failed")
    }
}

pub fn scrape_and_confirm(url: &str, confirm: bool) -> Result<String> {
    log::info!("Scraping data ...");
    let data = scrape(url).context("scrape failed")?;
    log::info!("Scraped data:\n{}", &data);

    confirm_if_needed(confirm)?;

    Ok(data)
}

pub fn extract_file_name_from_url(url: &str) -> Result<Option<String>> {
    let url = Url::parse(url)?;

    let file_name = url
        .path_segments()
        .map(|segments| segments.last())
        .flatten()
        .map(|item| item.to_string());

    Ok(file_name)
}

pub async fn download_file(src_url: &str) -> Result<String> {
    let downloads_dir = get_downloads_dir().ok_or(anyhow!("failed to find Downloads dir"))?;

    let file_name = extract_file_name_from_url(src_url)?
        .ok_or(anyhow!("failed to extract file name from url {}", src_url))?;

    let file = format!("{}/{}", &downloads_dir, file_name);
    ensure!(!file_exists(&file)?, "file {} already exists", file);

    download_data_to_file(src_url, &file).await?;
    log::debug!("Downloaded {} to {}", src_url, &file);

    Ok(file)
}
