use std::time::Duration;

use anyhow::{ensure, Context, Result};
use cookie::Cookie;
use fantoccini::{Client, ClientBuilder};
use full_page_screenshot::cdp_capture_full_page_screenshot;
use serde_json::json;
use tokio::fs as tokio_fs;

use rs_utils::{into_absolute_path, log, path_exists, Chromedriver};

use crate::scrape_result::ScrapeResult;
pub use crate::scrape_result::ScrapedData;

mod full_page_screenshot;
mod scrape_result;

#[derive(Default)]
pub struct ScraperOptions {
    pub debug: bool,
    pub manual: bool,
    pub emulate_mobile: bool,
    pub screenshot_file: Option<String>, // .png
}

impl ScraperOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn scrape(&self, url: &str) -> Result<Vec<ScrapeResult>> {
        let mut args = vec!["--incognito", "--no-sandbox", "--disable-dev-shm-usage"];

        let headless = !self.debug && !self.manual;
        if headless {
            args.push("--headless");
            args.push("window-size=1920,1080");
            args.push("--disable-gpu");

            args.push("user-agent=Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36");
        }

        if self.debug {
            args.push("--auto-open-devtools-for-tabs");
        }

        let capabilities = json!({
            "goog:chromeOptions": {
                "args": args,
                "mobileEmulation": if self.emulate_mobile {
                    json!({
                        "deviceName": "iPad Pro"
                    })
                } else {
                    json!({})
                },
            },
        });

        let mut chromedriver = Chromedriver::new()?;
        chromedriver.debug = self.debug;

        let mut chromedriver_process = chromedriver.spawn()?;

        chromedriver.wait_for_ready(10).await?;

        let client = ClientBuilder::rustls()
            .capabilities(
                capabilities
                    .as_object()
                    .context("must be an object")?
                    .clone(),
            )
            .connect(&chromedriver.get_url())
            .await
            .context("failed to connect to WebDriver")?;

        // TODO handle plain file downloads
        let result = self.run_browser_scraper(&client, url).await;

        if result.is_ok() {
            if let Err(err) = self.maybe_capture_sreenshot(&client).await {
                log::error!("Failed to save screenshot: {err}");
            };
        }

        if let Err(err) = client.close().await {
            log::error!("Failed to close webdriver client: {err}");
        }

        if let Err(err) = chromedriver_process.kill().await {
            log::error!("Failed to close webdriver: {err}");
        }

        result
    }

    async fn run_browser_scraper(&self, client: &Client, url: &str) -> Result<Vec<ScrapeResult>> {
        self.configure_timeouts(client).await?;

        client.goto(url).await.context("failed to open url")?;

        self.maybe_run_helpers(client, url)
            .await
            .context("failed to run scraper helpers")?;

        self.inject_browser_scraper(client).await?;

        let result = client
            .execute_async(
                r#"
                    const [url, manual, callback] = arguments;

                    window.originalURL = new URL(url);

                    window._doneCallback = () => {
                      callback(window._scraper.results);
                    };

                    if (manual) {
                      window._scraper.injectScraperUI();
                    } else {
                      try {
                        await window._scraper.scrape();
                      } finally {
                        window._doneCallback();
                      }
                    }
                "#,
                vec![json!(url), json!(self.manual)],
            )
            .await
            .context("failed to run scrape scenario")?;

        serde_json::from_value(result).context("failed to parse scrape results")
    }

    async fn maybe_run_helpers(&self, client: &Client, original_url: &str) -> Result<()> {
        let url = client.current_url().await?;

        // pass steam age check
        // https://store.steampowered.com/agecheck/app/814380/
        if url.host_str().unwrap_or_default() == "store.steampowered.com"
            && url.path().starts_with("/agecheck/")
        {
            let cookie = Cookie::build("birthtime", "628466401")
                .domain("store.steampowered.com")
                .path("/")
                .secure(false)
                .http_only(true)
                .same_site(cookie::SameSite::Strict)
                .finish();

            client
                .add_cookie(cookie)
                .await
                .context("failed to add steam age check cookie")?;

            client
                .goto(original_url)
                .await
                .context("failed to open original steam url")?;

            return Ok(());
        }

        Ok(())
    }

    async fn inject_browser_scraper(&self, client: &Client) -> Result<()> {
        client
            .execute(include_str!("../dist/browser-scraper.js"), vec![])
            .await
            .context("failed to inject browser-scraper.js")?;

        Ok(())
    }

    async fn maybe_capture_sreenshot(&self, client: &Client) -> Result<()> {
        let screenshot_file = if let Some(ref screenshot_file) = self.screenshot_file {
            screenshot_file
        } else {
            return Ok(());
        };

        let data = cdp_capture_full_page_screenshot(client)
            .await
            .context("failed to capture page screenshot")?;

        let file_path = into_absolute_path(screenshot_file, false)?;

        ensure!(
            !path_exists(&file_path),
            "Failed to save screenshot: {file_path} already exists"
        );

        tokio_fs::write(&file_path, data)
            .await
            .context("Failed to save screenshot")?;

        log::info!("Saved page screenshot into {file_path}");

        Ok(())
    }

    async fn configure_timeouts(&self, client: &Client) -> Result<()> {
        let mut timeouts = client
            .get_timeouts()
            .await
            .context("Failed to get session timeouts")?;

        if self.manual {
            timeouts.set_script(Some(Duration::from_secs(30 * 60)));
            timeouts.set_page_load(Some(Duration::from_secs(60)));
        } else {
            timeouts.set_script(Some(Duration::from_secs(60)));
            timeouts.set_page_load(Some(Duration::from_secs(30)));
        }

        client
            .update_timeouts(timeouts)
            .await
            .context("Failed to update session timeouts")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{ScrapeResult, ScraperOptions};

    async fn scrape(url: &str) -> Result<Vec<ScrapeResult>> {
        let scraper = ScraperOptions::new();

        scraper.scrape(url).await
    }

    #[tokio::test]
    #[ignore]
    async fn test_scrape_facebook_post() -> Result<()> {
        let result = scrape("https://www.facebook.com/theprodigyofficial/posts/pfbid02XeNwZbYFN8TeXtYrgSCRLPciWpfNWEu3HaUartDe7X5HUH8XGWeXYbHz8wKdREdml").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.permalink" => "[permalink]",
            "[].data.images[]" => "[permalink]"
        });

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_scrape_facebook_post_with_multiple_images() -> Result<()> {
        let result = scrape("https://www.facebook.com/theprodigyofficial/posts/pfbid0WVYZ4VTe9sddydcCNQGe7NUjajU92iVjM4TQYJgDpo14hy7zfHpQpdH5s2bWsoqul").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.permalink" => "[permalink]",
            "[].data.images[]" => "[permalink]"
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_yakaboo_book() -> Result<()> {
        let result =
            scrape("https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]"
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_steam_game() -> Result<()> {
        let result = scrape(
            "https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/",
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_myanimelist_movie() -> Result<()> {
        let result = scrape("https://myanimelist.net/anime/523/Tonari_no_Totoro").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_myanimelist_series() -> Result<()> {
        let result = scrape("https://myanimelist.net/anime/16498/Shingeki_no_Kyojin").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_film() -> Result<()> {
        let result = scrape("https://www.imdb.com/title/tt0133093/").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_series() -> Result<()> {
        let result = scrape("https://www.imdb.com/title/tt0098936/").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_mini_series() -> Result<()> {
        let result = scrape("https://www.imdb.com/title/tt8134186/").await?;

        insta::assert_json_snapshot!(result, {
            "[].data.coverURL" => "[permalink]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_image() -> Result<()> {
        let result =
            scrape("https://upload.wikimedia.org/wikipedia/en/7/7d/Lenna_%28test_image%29.png")
                .await?;

        insta::assert_json_snapshot!(result);

        Ok(())
    }
}
