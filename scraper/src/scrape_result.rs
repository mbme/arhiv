use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FacebookPostListItem {
    permalink: String,
    date: String,
    #[serde(rename = "dateISO")]
    date_iso: Option<String>,
    content: String,
    images: Vec<String>,
    videos: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FacebookMobilePostListItem {
    permalink: String,
    date: String,
    #[serde(rename = "dateISO")]
    date_iso: Option<String>,
    preview: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, tag = "typeName")]
pub enum ScrapedData {
    FacebookPost {
        permalink: String,
        date: String,
        #[serde(rename = "dateISO")]
        date_iso: Option<String>,
        content: String,
        images: Vec<String>,
    },
    FacebookPostList {
        posts: Vec<FacebookPostListItem>,
    },
    FacebookMobilePost {
        permalink: String,
        date: String,
        #[serde(rename = "dateISO")]
        date_iso: Option<String>,
        content: String,
    },
    FacebookMobilePostList {
        posts: Vec<FacebookMobilePostListItem>,
    },
    #[serde(rename_all = "camelCase")]
    IMDBFilm {
        title: String,
        #[serde(rename = "coverURL")]
        cover_url: String,
        release_date: String,
        original_language: String,
        countries_of_origin: String,
        creators: String,
        cast: String,
        seasons: Option<u8>,
        episodes: Option<u8>,
        duration: String,
        description: String,
    },
    #[serde(rename_all = "camelCase")]
    MyAnimeListAnime {
        title: String,
        #[serde(rename = "coverURL")]
        cover_url: String,
        release_date: String,
        creators: String,
        duration: String,
        description: String,
    },
    #[serde(rename_all = "camelCase")]
    SteamGame {
        #[serde(rename = "coverURL")]
        cover_url: String,
        name: String,
        release_date: String,
        developers: String,
        description: String,
    },
    #[serde(rename_all = "camelCase")]
    YakabooBook {
        #[serde(rename = "coverURL")]
        cover_url: String,
        title: String,
        authors: String,
        publication_date: String,
        description: String,
        translators: String,
        publisher: String,
        pages: u32,
        language: String,
    },
    Image {
        #[serde(rename = "imageURL")]
        image_url: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScrapeResult {
    pub url: String,
    pub original_url: Option<String>,
    pub scraper_name: Option<String>,
    pub data: Option<ScrapedData>,
    pub error: Option<String>,
}
