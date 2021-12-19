use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use dialoguer::{theme::ColorfulTheme, Confirm};

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::{log, run_js_script};

#[async_trait]
pub trait Importer {
    fn get_name(&self) -> &str;

    fn can_import(&self, url: &str) -> bool;

    async fn import(&self, url: &str, arhiv: &Arhiv, confirm: bool) -> Result<Document>;
}

fn scrape(args: Vec<&str>) -> Result<String> {
    let script = include_str!("../dist/bundle.js");

    run_js_script(script, args)
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
    }

    bail!("confirmation failed")
}

pub fn scrape_and_confirm(url: &str, confirm: bool) -> Result<String> {
    log::info!("Scraping data ...");
    let data = scrape(vec![url]).context("scrape failed")?;
    log::info!("Scraped data:\n{}", &data);

    confirm_if_needed(confirm)?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use serde_json::Value;

    use super::scrape;

    fn scrape_and_parse(url: &str) -> Result<Value> {
        let result = scrape(vec![url, "production"])?;

        serde_json::from_str(&result).context("failed to parse")
    }

    #[test]
    #[ignore]
    fn test_scrape_book() -> Result<()> {
        let result =
            scrape_and_parse("https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html")?;

        insta::assert_json_snapshot!("test_scrape_book", result);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_scrape_film() -> Result<()> {
        let result = scrape_and_parse("https://www.imdb.com/title/tt0133093/")?;

        insta::assert_json_snapshot!("test_scrape_film", result);

        Ok(())
    }
}
