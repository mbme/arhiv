#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::{Arhiv, Config};
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use rs_utils::log::setup_logger_with_level;

#[tokio::main]
async fn main() {
    if cfg!(not(feature = "production-mode")) {
        println!("DEBUG MODE");
    }

    let matches = App::new("arhiv")
        .subcommand(
            SubCommand::with_name("init").about("Initialize Arhiv instance on local machine"),
        )
        .subcommand(SubCommand::with_name("status").about("Print current status"))
        .subcommand(SubCommand::with_name("config").about("Print config"))
        .subcommand(SubCommand::with_name("sync").about("Sync changes"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!())
        .get_matches();

    setup_logger_with_level(matches.occurrences_of("verbose"));

    match matches.subcommand() {
        ("init", Some(_)) => {
            Arhiv::create(Config::must_read().0).expect("must be able to create arhiv");
        }
        ("status", Some(_)) => {
            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!("{}", status);
            // FIXME print number of unused temp attachments
        }
        ("config", Some(_)) => {
            let (config, path) = Config::must_read();
            println!("Arhiv config {}:", path);
            println!(
                "{}",
                serde_json::to_string_pretty(&config).expect("must be able to serialize config")
            );
        }
        ("sync", Some(_)) => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        _ => unreachable!(),
    }
}
