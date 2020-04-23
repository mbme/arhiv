#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::arhiv::Arhiv;
use arhiv::config::Config;
use clap::{crate_version, App};

fn get_arhiv() -> Arhiv {
    Arhiv::open(Config::read().unwrap()).expect("must be able to open arhiv")
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
            Arhiv::create(Config::read().unwrap()).expect("must be able to create arhiv");
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
