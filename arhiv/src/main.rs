#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use arhiv::utils::is_production_mode;
use arhiv::{start_server, Arhiv, Config};
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use log::LevelFilter;

#[tokio::main]
async fn main() {
    if !is_production_mode() {
        println!("DEBUG MODE");
    }

    let matches = App::new("arhiv")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize Arhiv instance on local machine")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .setting(AppSettings::DisableHelpSubcommand)
                .setting(AppSettings::DeriveDisplayOrder)
                .subcommand(
                    SubCommand::with_name("prime").about("Initialize Prime Arhiv on local machine"),
                )
                .subcommand(
                    SubCommand::with_name("replica")
                        .about("Initialize Replica Arhiv on local machine"),
                ),
        )
        .subcommand(SubCommand::with_name("status").about("Print current status"))
        .subcommand(SubCommand::with_name("prime-server").about("Run prime server"))
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
        ("init", Some(subcommand_matches)) => match subcommand_matches.subcommand() {
            ("prime", Some(_)) => {
                Arhiv::create(true, Config::must_read())
                    .expect("must be able to create Prime arhiv");
            }
            ("replica", Some(_)) => {
                Arhiv::create(false, Config::must_read())
                    .expect("must be able to create Replica arhiv");
            }
            _ => unreachable!(),
        },
        ("status", Some(_)) => {
            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!(
                "{} Arhiv (rev {}) on {}",
                if status.is_prime { "Prime" } else { "Replica" },
                status.rev,
                status.root_dir,
            );
            println!(
                "  Documents: {} committed, {} staged",
                status.committed_documents, status.staged_documents
            );
            println!(
                "Attachments: {} committed, {} staged",
                status.committed_attachments, status.staged_attachments
            );
            // FIXME print number of unused temp attachments
        }
        ("prime-server", Some(_)) => {
            let (join_handle, _, _) = start_server(Arhiv::must_open());

            join_handle.await.expect("must join");
        }
        ("sync", Some(_)) => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        _ => unreachable!(),
    }
}
