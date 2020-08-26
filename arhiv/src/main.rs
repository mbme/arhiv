#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::{start_server, Arhiv, Config};
use clap::{crate_version, App};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut app = App::new("arhiv")
        .subcommand(App::new("init").about("Initialize arhiv on local machine"))
        .subcommand(App::new("status").about("Print current status"))
        .subcommand(App::new("changes").about("List changes"))
        .subcommand(App::new("prime-server").about("Run prime server"))
        .subcommand(App::new("sync").about("Sync changes"))
        .version(crate_version!());
    // FIXME verbose flag to enable debug logs

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            Arhiv::create(Config::must_read()).expect("must be able to create arhiv");
        }
        ("status", Some(_)) => {
            // FIXME print root dir
            // FIXME print number of unused temp attachments
            println!(
                "{}",
                Arhiv::must_open()
                    .get_status()
                    .expect("must be able to get status")
            );
        }
        ("changes", Some(_)) => {
            // FIXME implement
        }
        ("prime-server", Some(_)) => {
            let (join_handle, _) = start_server(Arhiv::must_open());

            join_handle.await.expect("must join");
        }
        ("sync", Some(_)) => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        _ => app.print_help().unwrap(),
    }
}
