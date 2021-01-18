#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::sync::Arc;

use arhiv::{start_server, Arhiv, Config};
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use log::LevelFilter;

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
        .subcommand(SubCommand::with_name("server").about("Run prime server"))
        .subcommand(SubCommand::with_name("sync").about("Sync changes"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .global(true)
                .help("Enable debug logs"),
        )
        .arg(
            Arg::with_name("trace")
                .long("trace")
                .global(true)
                .help("Enable trace logs"),
        )
        .version(crate_version!())
        .get_matches();

    // init logger
    let mut log_level = LevelFilter::Info;
    if matches.occurrences_of("debug") > 0 {
        log_level = LevelFilter::Debug;
    }
    if matches.occurrences_of("trace") > 0 {
        log_level = LevelFilter::Trace;
    }
    env_logger::builder().filter(None, log_level).init();

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
        ("server", Some(_)) => {
            let arhiv = Arc::new(Arhiv::must_open());
            if !arhiv
                .get_status()
                .expect("must be able to get status")
                .is_prime
            {
                panic!("server must be started on prime instance");
            }

            let (join_handle, _, _) = start_server(arhiv);

            join_handle.await.expect("must join");
        }
        ("sync", Some(_)) => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        _ => unreachable!(),
    }
}
