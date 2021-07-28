use arhiv_core::Arhiv;
use arhiv_import::run_app;
use rs_utils::log;

pub fn main() {
    log::setup_logger();

    let arhiv = Arhiv::must_open();

    run_app(
        "https://www.yakaboo.ua/ua/soviet-modernism-brutalism-post-modernism-buildings.html",
        &arhiv,
        true,
    )
    .expect("failed to import");
}
