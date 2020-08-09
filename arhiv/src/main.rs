#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::config::Config;
use arhiv::Arhiv;
use clap::{crate_version, App};

fn main() {
    env_logger::init();

    let mut app = App::new("arhiv")
        .subcommand(App::new("init").about("Initialize arhiv on local machine"))
        .subcommand(App::new("status").about("Print current status"))
        .subcommand(App::new("server").about("Run prime server"))
        .subcommand(App::new("commit").about("Commit changes"))
        .version(crate_version!());

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Arhiv::create(Config::must_read()).expect("must be able to create arhiv");
        }
        ("status", Some(_)) => {
            println!(
                "{:?}",
                Arhiv::must_open()
                    .get_status()
                    .expect("must be able to get status")
            );
        }
        ("server", Some(_)) => {
            Arhiv::must_open().start_server();
        }
        ("commit", Some(_)) => {
            Arhiv::must_open().commit().expect("must commit");
        }
        _ => app.print_help().unwrap(),
    }
}
