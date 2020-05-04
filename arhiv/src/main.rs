#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::arhiv::Arhiv;
use arhiv::config::Config;
use arhiv::server::start_server;
use clap::{crate_version, App};

fn get_arhiv() -> Arhiv {
    Arhiv::open(Config::read().unwrap()).expect("must be able to open arhiv")
}

fn main() {
    env_logger::init();

    let mut app = App::new("arhiv")
        .subcommand(App::new("init").about("Initialize arhiv on local machine"))
        .subcommand(App::new("rev").about("Print current revision"))
        .subcommand(App::new("server").about("Run primary server"))
        .subcommand(App::new("sync").about("Trigger sync with primary server"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Arhiv::create(Config::read().unwrap()).expect("must be able to create arhiv");
        }
        ("rev", Some(_)) => {
            println!(
                "revision: {}",
                get_arhiv().get_rev().expect("must be able to get revision")
            );
        }
        ("server", Some(_)) => {
            start_server(get_arhiv());
        }
        _ => app.print_help().unwrap(),
    }
}
