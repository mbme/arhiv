use arhiv_replica::prime::{Prime, PrimeConfig};
use clap::{crate_version, App};
use std::env;
use std::fs;

fn read_config() -> PrimeConfig {
    let path = &format!(
        "{}/arhiv-prime.json",
        env::var("CARGO_MANIFEST_DIR").expect("env var CARGO_MANIFEST_DIR must be set")
    );

    fs::read_to_string(path)
        .expect(&format!("must be able to read arhiv config at {}", path))
        .parse()
        .expect(&format!("must be able to parse arhiv config at {}", path))
}

fn main() {
    env_logger::init();

    let mut app = App::new("arhiv-prime")
        .subcommand(App::new("init").about("Initialize prime server"))
        .subcommand(App::new("start").about("Run prime server"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Prime::create(read_config()).expect("must be able to create prime");
        }
        ("start", Some(_)) => {
            let prime = Prime::open(read_config()).expect("must be able to open prime");
            prime.start_server();
        }
        _ => app.print_help().unwrap(),
    }
}
