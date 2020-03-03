use arhiv_replica::replica::{Replica, ReplicaConfig};
use clap::{crate_version, App};
use std::env;
use std::fs;

fn read_config() -> ReplicaConfig {
    let path = &format!(
        "{}/arhiv.json",
        env::var("CARGO_MANIFEST_DIR").expect("env var CARGO_MANIFEST_DIR must be set")
    );

    fs::read_to_string(path)
        .expect(&format!("must be able to read arhiv config at {}", path))
        .parse()
        .expect(&format!("must be able to parse arhiv config at {}", path))
}

fn main() {
    let mut app = App::new("arhiv-replica")
        .subcommand(App::new("init").about("Initialize replica on local machine"))
        .subcommand(App::new("sync").about("Trigger sync with primary server"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Replica::create(read_config()).expect("must be able to create replica");
        }
        _ => app.print_help().unwrap(),
    }
}
