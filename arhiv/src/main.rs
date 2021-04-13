#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};

use arhiv::{
    server::{start_prime_server, start_ui_server},
    Arhiv, Config,
};
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use rs_utils::log::{setup_logger_with_level, setup_server_logger};

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
        .subcommand(
            SubCommand::with_name("ui")
                .about("Show arhiv UI")
                .arg(
                    Arg::with_name("open")
                        .long("open")
                        .takes_value(true)
                        .help("Open app using provided browser or $BROWSER env variable"),
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .takes_value(true)
                        .help("Listen on specific port"),
                ),
        )
        .subcommand(SubCommand::with_name("server").about("Run prime server"))
        .subcommand(
            SubCommand::with_name("backup")
                .about("Backup arhiv data")
                .arg(
                    Arg::with_name("backup_dir")
                        .help("Directory to save backup")
                        .index(1)
                        .required(true),
                ),
        )
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

    match matches.subcommand() {
        ("init", Some(_)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            Arhiv::create(Config::must_read().0).expect("must be able to create arhiv");
        }
        ("status", Some(_)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!("{}", status);
            // FIXME print number of unused temp attachments
        }
        ("config", Some(_)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            let (config, path) = Config::must_read();
            println!("Arhiv config {}:", path);
            println!(
                "{}",
                serde_json::to_string_pretty(&config).expect("must be able to serialize config")
            );
        }
        ("sync", Some(_)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            Arhiv::must_open().sync().await.expect("must sync");
        }
        ("ui", Some(matches)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            let browser = {
                if let Some(browser) = matches.value_of("open") {
                    browser.to_string()
                } else {
                    env::var("BROWSER").expect("failed to read BROWSER env variable")
                }
            };

            let port: Option<u16> = matches
                .value_of("port")
                .map(|value| value.parse().expect("port must be valid u16"));

            let (join_handle, addr) = start_ui_server(port).await;

            Command::new(&browser)
                .arg(format!("http://{}", addr))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect(&format!("failed to run browser {}", browser));

            join_handle.await.expect("must join");
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

            let log_file = Path::new(arhiv.config.get_root_dir()).join("arhiv-server.log");
            setup_server_logger(log_file);

            let (join_handle, _, _) = start_prime_server(arhiv);

            join_handle.await.expect("must join");
        }
        ("backup", Some(matches)) => {
            setup_logger_with_level(matches.occurrences_of("verbose"));

            let arhiv = Arc::new(Arhiv::must_open());

            let backup_dir = matches
                .value_of("backup_dir")
                .expect("backup_dir must be present");

            arhiv.backup(backup_dir).expect("must be able to backup");
        }
        _ => unreachable!(),
    }
}
