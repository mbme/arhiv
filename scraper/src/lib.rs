#![deny(clippy::all)]

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{log, run_command_with_envs, Chromium, NodeJS, TempFile};

fn get_script_temp_file() -> Result<TempFile> {
    let script = include_str!("../dist/node-scraper.js");

    // TODO use "shared memory file" shm_open
    let temp_file = TempFile::new_with_details("scrape-script-", ".js");

    temp_file.write(script)?;

    Ok(temp_file)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScrapeResult {
    url: String,
    original_url: Option<String>,
    scraper_name: Option<String>,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

pub struct Scraper {
    chrome_bin_path: String,
    debug: bool,
    manual: bool,
    mobile: bool,
}

impl Scraper {
    pub fn new() -> Result<Self> {
        NodeJS::check()?;

        let chromium = Chromium::check()?;

        Ok(Scraper {
            chrome_bin_path: chromium.get_bin_path().to_string(),
            debug: false,
            manual: false,
            mobile: false,
        })
    }

    pub fn debug(&mut self) {
        self.debug = true;
    }

    pub fn manual(&mut self) {
        self.manual = true;
    }

    pub fn emulate_mobile(&mut self) {
        self.mobile = true;
    }

    pub fn scrape(&self, url: &str) -> Result<Vec<ScrapeResult>> {
        log::info!("Scraping data from '{}'", url);

        let script_temp_file = get_script_temp_file()?;

        let mut envs = HashMap::new();
        envs.insert("CHROME_BIN_PATH", self.chrome_bin_path.as_str());

        let mut args = vec!["--title", "scraper", &script_temp_file.path, url];

        if self.debug {
            args.push("--debug");
        }

        if self.manual {
            args.push("--manual")
        }

        if self.mobile {
            args.push("--mobile")
        }

        let result = run_command_with_envs("node", args, envs).context("scrape failed")?;

        // TODO handle plain file downloads

        serde_json::from_str(&result).context("failed to parse JSON")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{ScrapeResult, Scraper};

    fn scrape(url: &str) -> Result<Vec<ScrapeResult>> {
        let scraper = Scraper::new()?;

        scraper.scrape(url)
    }

    #[test]
    #[ignore]
    fn test_scrape_facebook_post() -> Result<()> {
        let result = scrape("https://www.facebook.com/theprodigyofficial/posts/pfbid02XeNwZbYFN8TeXtYrgSCRLPciWpfNWEu3HaUartDe7X5HUH8XGWeXYbHz8wKdREdml")?;

        insta::assert_json_snapshot!(result, {
            "[].data.permalink" => "[permalink]"
        });

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_scrape_facebook_post_with_multiple_images() -> Result<()> {
        let result = scrape("https://www.facebook.com/theprodigyofficial/posts/pfbid0WVYZ4VTe9sddydcCNQGe7NUjajU92iVjM4TQYJgDpo14hy7zfHpQpdH5s2bWsoqul")?;

        insta::assert_json_snapshot!(result, {
            "[].data.permalink" => "[permalink]"
        });

        Ok(())
    }
}
