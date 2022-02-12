use std::env;

use arhiv_core::Arhiv;
use arhiv_scraper::Scraper;
use rs_utils::log;

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    let args = env::args().collect::<Vec<_>>();
    let url = args.get(1).unwrap();

    let arhiv = Arhiv::must_open();

    let mut scraper = Scraper::new(&arhiv).unwrap();
    scraper.debug();

    let documents = scraper.scrape(url).await.expect("failed to run scrapers");

    for document in documents {
        println!("{:#?}", document);
    }
}
