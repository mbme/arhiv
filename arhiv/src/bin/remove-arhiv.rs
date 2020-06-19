use arhiv::config::Config;
use arhiv::utils::dir_exists;
use std::fs;

fn main() {
    env_logger::init();

    let config = Config::must_read();

    if dir_exists(&config.arhiv_root).unwrap() {
        fs::remove_dir_all(&config.arhiv_root).expect("must be able to remove arhiv");
        log::info!("removed arhiv {}", &config.arhiv_root);
    } else {
        log::info!("arhiv {} doesn't exist", &config.arhiv_root);
    }
}
