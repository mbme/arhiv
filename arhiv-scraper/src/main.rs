use std::env;

use arhiv_core::Arhiv;
use rs_utils::{log, EnvCapabilities};
use arhiv_scraper::scrape;

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    let args = env::args().collect::<Vec<_>>();
    let url = args.get(1).unwrap();

    let arhiv = Arhiv::must_open();

    let capabilities = EnvCapabilities::must_check();

    scrape(&arhiv, &capabilities, url, false)
        .await
        .expect("failed to run scrapers");
}
