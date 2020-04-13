#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::Arhiv;
use clap::{crate_version, App};
use config::Config;
use std::env;
use std::fs;

mod arhiv;
mod config;
mod entities;
// mod server;
mod utils;

fn read_config() -> Config {
    let path = &format!(
        "{}/arhiv.json",
        env::var("CARGO_MANIFEST_DIR").expect("env var CARGO_MANIFEST_DIR must be set")
    );

    fs::read_to_string(path)
        .expect(&format!("must be able to read arhiv config at {}", path))
        .parse()
        .expect(&format!("must be able to parse arhiv config at {}", path))
}

fn get_arhiv() -> Arhiv {
    Arhiv::open(read_config()).expect("must be able open create arhiv")
}

fn main() {
    env_logger::init();

    let mut app = App::new("arhiv")
        .subcommand(App::new("init").about("Initialize arhiv on local machine"))
        .subcommand(App::new("rev").about("Print current revision"))
        .subcommand(App::new("start").about("Run primary server"))
        .subcommand(App::new("sync").about("Trigger sync with primary server"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Arhiv::create(read_config()).expect("must be able to create arhiv");
        }
        ("rev", Some(_)) => {
            println!("revision: {}", get_arhiv().get_rev().unwrap());
        }
        ("start", Some(_)) => {
            // let arhiv = Arhiv::open(read_config()).expect("must be able to open arhiv");
            // arhiv.start_server();
        }
        _ => app.print_help().unwrap(),
    }
}
