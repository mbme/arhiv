#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{env, process, sync::Arc};

use clap::{crate_version, App, AppSettings, Arg, SubCommand};

use arhiv_core::{
    apply_migrations,
    entities::{Document, DocumentData, Id},
    get_schema,
    prime_server::start_prime_server,
    Arhiv, Config,
};
use arhiv_ui3::start_ui_server;
use rs_utils::log::setup_logger_with_level;

#[tokio::main]
async fn main() {
    if cfg!(not(feature = "production-mode")) {
        println!("DEBUG MODE");
    }

    let matches = App::new("arhiv")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize Arhiv instance on local machine")
                .arg(
                    Arg::with_name("arhiv_id")
                        .long("arhiv_id")
                        .required(true)
                        .index(1)
                        .help("Arhiv id to use"),
                )
                .arg(
                    Arg::with_name("prime")
                        .long("prime")
                        .display_order(1)
                        .help("Initialize prime instance"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync") //
                .about("Sync changes"),
        )
        .subcommand(
            SubCommand::with_name("apply-migrations")
                .about("Upgrade arhiv db schema to latest version"),
        )
        .subcommand(
            SubCommand::with_name("backup") //
                .about("Backup arhiv data"),
        )
        .subcommand(
            SubCommand::with_name("ui-server")
                .about("Run arhiv UI server")
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .takes_value(true)
                        .default_value("23421")
                        .help("Listen on specific port"),
                ),
        )
        .subcommand(
            SubCommand::with_name("prime-server")
                .about("Run prime server")
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .takes_value(true)
                        .default_value("23420")
                        .help("Listen on specific port"),
                ),
        )
        .subcommand(
            SubCommand::with_name("status") //
                .about("Print current status"),
        )
        .subcommand(
            SubCommand::with_name("config") //
                .about("Print config")
                .arg(
                    Arg::with_name("template")
                        .short("t")
                        .long("template")
                        .display_order(1)
                        .help("Prints config template"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get document by id")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .help("id of the document"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add new document")
                .arg(
                    Arg::with_name("document_type")
                        .required(true)
                        .possible_values(&get_schema().get_document_types(true))
                        .index(1)
                        .help("One of known document types"),
                )
                .arg(
                    Arg::with_name("data")
                        .required(true)
                        .index(2)
                        .help("JSON object with document props"),
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

    setup_logger_with_level(matches.occurrences_of("verbose"));

    match matches.subcommand() {
        ("init", Some(matches)) => {
            let arhiv_id: String = matches
                .value_of("arhiv_id")
                .expect("arhiv_id must be present")
                .to_string();

            let prime = matches.is_present("prime");

            Arhiv::create(Config::must_read().0, arhiv_id, prime)
                .expect("must be able to create arhiv");
        }
        ("status", _) => {
            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!("{}", status);
            // FIXME print number of unused temp attachments
        }
        ("config", Some(matches)) => {
            if matches.is_present("template") {
                print!("{}", include_str!("../../arhiv.json.template"));
                return;
            }

            let (config, path) = Config::must_read();
            println!("Arhiv config {}:", path);
            println!(
                "{}",
                serde_json::to_string_pretty(&config).expect("must be able to serialize config")
            );
        }
        ("sync", _) => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        ("get", Some(matches)) => {
            let id: Id = matches.value_of("id").expect("id must be present").into();

            let arhiv = Arhiv::must_open();

            let document = arhiv
                .get_document(&id)
                .expect("must be able to query for a document");

            if let Some(document) = document {
                serde_json::to_writer_pretty(std::io::stdout(), &document)
                    .expect("must be able to serialize document");
            } else {
                eprintln!("Document with id '{}' not found", &id);
                process::exit(1);
            }
        }
        ("add", Some(matches)) => {
            let document_type: &str = matches
                .value_of("document_type")
                .expect("document_type must be present");

            let data: &str = matches.value_of("data").expect("data must be present");
            let data: DocumentData =
                serde_json::from_str(data).expect("data must be a JSON object");

            let document = Document::new_with_data(document_type, data);
            let id = document.id.clone();

            let arhiv = Arhiv::must_open();

            arhiv
                .stage_document(document)
                .expect("must be able to stage document");

            println!("{}", id);
        }
        ("ui-server", Some(matches)) => {
            let port: u16 = matches
                .value_of("port")
                .map(|value| value.parse().expect("port must be valid u16"))
                .expect("port is missing");

            start_ui_server(port).await;
        }
        ("prime-server", Some(matches)) => {
            let arhiv = Arc::new(Arhiv::must_open());

            if !arhiv
                .get_status()
                .expect("must be able to get status")
                .db_status
                .is_prime
            {
                panic!("server must be started on prime instance");
            }

            let port: u16 = matches
                .value_of("port")
                .map(|value| value.parse().expect("port must be valid u16"))
                .expect("port is missing");

            let (join_handle, _, _) = start_prime_server(arhiv, port);

            join_handle.await.expect("must join");
        }
        ("backup", _) => {
            let arhiv = Arhiv::must_open();

            arhiv.backup().expect("must be able to backup");
        }
        ("apply-migrations", _) => {
            let config = Config::must_read().0;

            apply_migrations(config.arhiv_root)
                .expect("must be able to apply migrations to arhiv db");
        }
        _ => unreachable!(),
    }
}
