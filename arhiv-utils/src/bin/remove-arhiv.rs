use arhiv::Config;
use rs_utils::{
    dir_exists,
    log::{info, setup_logger},
};
use std::fs;

fn main() {
    setup_logger();

    let config = Config::must_read().0;

    if dir_exists(&config.arhiv_root).unwrap() {
        fs::remove_dir_all(&config.arhiv_root).expect("must be able to remove arhiv");
        info!("removed arhiv {}", config.arhiv_root);
    } else {
        info!("arhiv {} doesn't exist", config.arhiv_root);
    }
}
