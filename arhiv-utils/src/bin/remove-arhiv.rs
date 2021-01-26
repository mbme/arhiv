use arhiv::Config;
use rs_utils::{dir_exists, setup_logger};
use std::fs;
use tracing::info;

fn main() {
    setup_logger();

    let config = Config::must_read().0;

    if dir_exists(config.get_root_dir()).unwrap() {
        fs::remove_dir_all(config.get_root_dir()).expect("must be able to remove arhiv");
        info!("removed arhiv {}", config.get_root_dir());
    } else {
        info!("arhiv {} doesn't exist", config.get_root_dir());
    }
}
