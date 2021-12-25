use std::env;

use arhiv_core::Arhiv;

use arhiv_import::scrape;
use rs_utils::log;

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    let args = env::args().collect::<Vec<_>>();
    let url = args.get(1).unwrap();

    let arhiv = Arhiv::must_open();

    scrape(&arhiv, url, false, true)
        .await
        .expect("failed to run importers");
}
