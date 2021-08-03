#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env,
    process::{self, Command, Stdio},
    sync::Arc,
};

use clap::{crate_version, App, AppSettings, Arg, SubCommand};

use arhiv_core::{
    apply_migrations,
    entities::{Document, DocumentData, Id},
    get_schema,
    prime_server::start_prime_server,
    Arhiv, Config,
};
use arhiv_import::ArhivImport;
use arhiv_ui3::start_ui_server;
use rs_utils::{
    into_absolute_path,
    log::{self, setup_logger_with_level},
};

#[tokio::main]
async fn main() {
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
            SubCommand::with_name("ui-server") //
                .about("Run arhiv UI server"),
        )
        .subcommand(
            SubCommand::with_name("ui-open") //
                .about("Open document in UI")
                .arg(
                    Arg::with_name("id")
                        .index(1)
                        .required(true)
                        .help("document id to open"),
                )
                .arg(
                    Arg::with_name("browser")
                        .long("browser")
                        .takes_value(true)
                        .min_values(0)
                        .env("BROWSER")
                        .help("Open using provided browser or fall back to $BROWSER env variable"),
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
        .subcommand(
            SubCommand::with_name("attach")
                .about("Add new attachment. Will hard link or copy file to arhiv.")
                .arg(
                    Arg::with_name("file_path")
                        .required(true)
                        .index(1)
                        .help("Absolute path to file to save"),
                )
                .arg(
                    Arg::with_name("move_file")
                        .short("m")
                        .help("Move file to arhiv"),
                ),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Scrape data and create document")
                .arg(
                    Arg::with_name("url") //
                        .required(true)
                        .index(1)
                        .help("url to scrape"),
                )
                .arg(
                    Arg::with_name("skip_confirmation")
                        .long("skip_confirmation")
                        .help("Import scraped data without confirmation"),
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

            let mut document = Document::new_with_data(document_type, data);

            let arhiv = Arhiv::must_open();

            arhiv
                .stage_document(&mut document)
                .expect("must be able to stage document");

            println!(
                "{} {}",
                document.id,
                document_url(&document.id, arhiv.config.ui_server_port)
            );
        }
        ("attach", Some(matches)) => {
            let file_path: &str = matches
                .value_of("file_path")
                .expect("file_path must be present");

            let file_path =
                into_absolute_path(file_path).expect("failed to convert path to absolute");

            let move_file: bool = matches.is_present("move_file");

            let arhiv = Arhiv::must_open();

            let attachment = arhiv
                .add_attachment(&file_path, move_file)
                .expect("must be able to save attachment");

            println!(
                "{} {}",
                attachment.id,
                document_url(&attachment.id, arhiv.config.ui_server_port)
            );
        }
        ("import", Some(matches)) => {
            let url: &str = matches.value_of("url").expect("url must be present");

            let skip_confirmation: bool = matches.is_present("skip_confirmation");

            let arhiv = Arhiv::must_open();
            let port = arhiv.config.ui_server_port;
            let mut importer = ArhivImport::new(arhiv);

            importer.confirm(!skip_confirmation);

            let id = importer
                .import(url)
                .await
                .expect("failed to import document");

            println!("{} {}", id, document_url(&id, port));
        }
        ("ui-server", _) => {
            start_ui_server().await;
        }
        ("ui-open", Some(matches)) => {
            let id: Id = matches.value_of("id").expect("id must be present").into();

            let port = Config::must_read().0.ui_server_port;

            let browser = matches
                .value_of("browser")
                .expect("either browser must be specified or $BROWSER env var must be set");

            log::info!("Opening document {} UI in {}", id, browser);

            Command::new(&browser)
                .arg(document_url(&id, port))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect(&format!("failed to run browser {}", browser));
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
                .expect("port is missing or invalid");

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

fn document_url(id: &Id, port: u16) -> String {
    format!("http://localhost:{}/documents/{}", port, id)
}
