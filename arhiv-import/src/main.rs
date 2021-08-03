use arhiv_core::Arhiv;
use arhiv_import::ArhivImport;
use rs_utils::log;

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    let arhiv = Arhiv::must_open();

    let importer = ArhivImport::new(arhiv);

    importer
        .import(
            "https://www.yakaboo.ua/ua/soviet-modernism-brutalism-post-modernism-buildings.html",
        )
        .await
        .expect("failed to import");
}
