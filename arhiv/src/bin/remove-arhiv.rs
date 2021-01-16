use arhiv::Config;
use rs_utils::dir_exists;
use std::fs;

fn main() {
    env_logger::init();

    let config = Config::must_read();

    if dir_exists(config.get_root_dir()).unwrap() {
        fs::remove_dir_all(config.get_root_dir()).expect("must be able to remove arhiv");
        log::info!("removed arhiv {}", config.get_root_dir());
    } else {
        log::info!("arhiv {} doesn't exist", config.get_root_dir());
    }
}
