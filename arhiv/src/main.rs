#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::sync::Arc;

use arhiv::{start_server, Arhiv, Config};
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use rs_utils::get_log_level;
use tracing::{debug, trace, Level};

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
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .global(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!())
        .get_matches();

    let log_level = get_log_level(matches.occurrences_of("verbose"));
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(log_level)
            .finish(),
    )
    .expect("setting default subscriber failed");

    if log_level == Level::DEBUG {
        debug!("DEBUG output enabled.");
    }
    if log_level == Level::TRACE {
        trace!("TRACE output enabled.");
    }

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
                .db_status
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
