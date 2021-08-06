use std::env;

use arhiv_core::Arhiv;
use arhiv_import::ArhivImport;
use rs_utils::log;

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    let args = env::args().collect::<Vec<_>>();
    let ref url = args.get(1).unwrap();

    let arhiv = Arhiv::must_open();

    let importer = ArhivImport::new(arhiv);

    importer.import(url).await.expect("failed to import");
}
