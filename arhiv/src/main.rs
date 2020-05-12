#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::config::Config;
use arhiv::server::start_server;
use arhiv::Arhiv;
use clap::{crate_version, App};

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
            Arhiv::create(Config::must_read()).expect("must be able to create arhiv");
        }
        ("rev", Some(_)) => {
            println!(
                "revision: {}",
                Arhiv::must_open()
                    .get_rev()
                    .expect("must be able to get revision")
            );
        }
        ("server", Some(_)) => {
            start_server(Arhiv::must_open());
        }
        _ => app.print_help().unwrap(),
    }
}
