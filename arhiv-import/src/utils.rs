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
    } else {
        bail!("confirmation failed")
    }
}

pub fn scrape_and_confirm(url: &str, confirm: bool) -> Result<String> {
    log::info!("Scraping data ...");
    let data = scrape(vec![url]).context("scrape failed")?;
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

#[cfg(test)]
mod tests {
    use anyhow::*;
    use serde_json::{json, Value};

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

        assert_eq!(
            result,
            json!({
                "ISBN":"9781529039436",
                "authors":"Тед Чан",
                "cover_src":"https://img.yakaboo.ua/media/catalog/product/cache/1/image/398x565/31b681157c4c1a5551b0db4896e7972f/i/m/img_53701.jpg",
                "description":"Includes 'Story of Your Life' the basis for the major motion picture Arrival, starring Amy Adams, Forest Whitaker, Jeremy Renner, and directed by Denis Villeneuve.\n\nWith his masterful first collection, multiple-award-winning author Ted Chiang deftly blends human emotion and scientific rationalism in eight remarkably diverse stories, all told in his trademark precise and evocative prose.\n\nFrom a soaring Babylonian tower that connects a flat Earth with the firmament above, to a world where angelic visitations are a wondrous and terrifying part of everyday life; from a neural modification that eliminates the appeal of physical beauty, to an alien language that challenges our very perception of time and reality. . . Chiang's rigorously imagined fantasia invites us to question our understanding of the universe and our place in it.",
                "language":"English",
                "pages":340,
                "publication_date":"2020",
                "publisher":"Picador",
                "title":"Книга «Stories of Your Life and Others» – Тед Чан"
            })
        );

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_scrape_film() -> Result<()> {
        let result = scrape_and_parse("https://www.imdb.com/title/tt0133093/")?;

        assert_eq!(
            result,
            json!({
                "cast":"Keanu Reeves, Laurence Fishburne, Carrie-Anne Moss",
                "countries_of_origin":"United States, Australia",
                "cover_src":"https://m.media-amazon.com/images/M/MV5BNzQzOTk3OTAtNDQ0Zi00ZTVkLWI0MTEtMDllZjNkYzNjNTc4L2ltYWdlXkEyXkFqcGdeQXVyNjU0OTQ0OTY@._V1_QL75_UX190_CR0,2,190,281_.jpg",
                "creators":"Lana Wachowski, Lilly Wachowski",
                "description":"Thomas A. Anderson is a man living two lives. By day he is an average computer programmer and by night a hacker known as Neo. Neo has always questioned his reality, but the truth is far beyond his imagination. Neo finds himself targeted by the police when he is contacted by Morpheus, a legendary computer hacker branded a terrorist by the government. As a rebel against the machines, Neo must confront the agents: super-powerful computer programs devoted to stopping Neo and the entire human rebellion.",
                "duration":"2h 16min",
                "is_series":"false",
                "original_language":"English",
                "release_date":"1999",
                "title":"The Matrix"
            })
        );

        Ok(())
    }
}
