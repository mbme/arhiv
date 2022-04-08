use std::{process, sync::Arc};

use clap::{crate_version, AppSettings, Arg, Command};
use clap_complete::{generate_to, Shell};

use arhiv_core::{
    definitions::get_standard_schema,
    entities::{Document, DocumentData, Id},
    prime_server::start_prime_server,
    Arhiv, Config,
};
use arhiv_scraper::Scraper;
use arhiv_ui3::start_ui_server;
use rs_utils::{into_absolute_path, log};

use crate::import::import_document_from_file;

#[allow(clippy::too_many_lines)]
#[must_use]
fn build_app() -> Command<'static> {
    Command::new("arhiv")
        .bin_name("arhiv")
        .subcommand(
            Command::new("init")
                .about("Initialize Arhiv instance on local machine")
                .arg(
                    Arg::new("arhiv_id")
                        .long("arhiv_id")
                        .required(true)
                        .index(1)
                        .help("Arhiv id to use"),
                )
                .arg(
                    Arg::new("prime")
                        .long("prime")
                        .display_order(1)
                        .help("Initialize prime instance"),
                ),
        )
        .subcommand(
            Command::new("sync") //
                .about("Sync changes"),
        )
        .subcommand(
            Command::new("backup") //
                .about("Backup arhiv data"),
        )
        .subcommand(
            Command::new("ui-server") //
                .about("Run arhiv UI server"),
        )
        .subcommand(
            Command::new("ui-open") //
                .about("Open document in UI")
                .arg(
                    Arg::new("id")
                        .index(1)
                        .required(true)
                        .help("document id to open"),
                )
                .arg(
                    Arg::new("browser")
                        .long("browser")
                        .takes_value(true)
                        .min_values(0)
                        .env("BROWSER")
                        .help("Open using provided browser or fall back to $BROWSER env variable"),
                ),
        )
        .subcommand(
            Command::new("prime-server").about("Run prime server").arg(
                Arg::new("port")
                    .long("port")
                    .takes_value(true)
                    .default_value("23420")
                    .help("Listen on specific port"),
            ),
        )
        .subcommand(
            Command::new("status") //
                .about("Print current status"),
        )
        .subcommand(
            Command::new("config") //
                .about("Print config")
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .display_order(1)
                        .help("Prints config template"),
                ),
        )
        .subcommand(
            Command::new("get")
                .about("Get document by id")
                .arg(Arg::new("id").required(true).help("id of the document")),
        )
        .subcommand(
            Command::new("add")
                .about("Add new document")
                .arg(
                    Arg::new("document_type")
                        .required(true)
                        .possible_values(get_standard_schema().get_document_types())
                        .index(1)
                        .help("One of known document types"),
                )
                .arg(Arg::new("subtype").help("Document subtype"))
                .arg(
                    Arg::new("data")
                        .required(true)
                        .index(2)
                        .help("JSON object with document props"),
                ),
        )
        .subcommand(
            Command::new("scrape")
                .about("Scrape remote resource and create document")
                .arg(
                    Arg::new("url") //
                        .required(true)
                        .index(1)
                        .help("url to scrape"),
                ),
        )
        .subcommand(
            Command::new("import")
                .about("Import files and create documents. Will hard link or copy files to Arhiv.")
                .arg(
                    Arg::new("document_type")
                        .required(true)
                        .possible_values(get_standard_schema().get_document_types())
                        .index(1)
                        .help("One of known document types"),
                )
                .arg(
                    Arg::new("file_path") //
                        .required(true)
                        .index(2)
                        .multiple_values(true)
                        .help("Files to import"),
                )
                .arg(Arg::new("move_file").short('m').help("Move file to arhiv")),
        )
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("verbose")
                .short('v')
                .multiple_occurrences(true)
                .global(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!())
}

#[allow(clippy::too_many_lines)]
pub async fn arhiv_cli() {
    let matches = build_app().get_matches();

    let verbose_count = matches.occurrences_of("verbose");
    match verbose_count {
        0 => log::setup_logger(),
        1 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    match matches.subcommand().expect("subcommand must be provided") {
        ("init", matches) => {
            let arhiv_id = matches
                .value_of("arhiv_id")
                .expect("arhiv_id must be present");

            let prime = matches.is_present("prime");

            let config = Config::must_read().0;
            let schema = get_standard_schema();

            Arhiv::create(config, schema, arhiv_id, prime).expect("must be able to create arhiv");
        }
        ("status", _) => {
            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!("{}", status);
            // FIXME print number of unused temp attachments
        }
        ("config", matches) => {
            if matches.is_present("template") {
                print!("{}", include_str!("../arhiv.json.template"));
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
        ("get", matches) => {
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
        ("add", matches) => {
            let document_type: &str = matches
                .value_of("document_type")
                .expect("document_type must be present");
            let subtype: &str = matches.value_of("subtype").unwrap_or_default();

            let data: &str = matches.value_of("data").expect("data must be present");
            let data: DocumentData =
                serde_json::from_str(data).expect("data must be a JSON object");

            let mut document = Document::new_with_data(document_type, subtype, data);

            let arhiv = Arhiv::must_open();

            let tx = arhiv.get_tx().expect("must open tx");

            tx.stage_document(&mut document)
                .expect("must be able to stage document");

            tx.commit().expect("must commit");

            let port = arhiv.get_config().ui_server_port;
            print_document(&document, port);
        }
        ("scrape", matches) => {
            let url: &str = matches.value_of("url").expect("url must be present");

            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().ui_server_port;

            let scraper = Scraper::new(&arhiv).expect("failed to create scraper");
            let documents = scraper.scrape(url).await.expect("failed to scrape");

            for document in documents {
                print_document(&document, port);
            }
        }
        ("import", matches) => {
            let document_type: &str = matches
                .value_of("document_type")
                .expect("document_type must be provided");

            let file_paths: Vec<&str> = matches
                .values_of("file_path")
                .expect("file_path must be provided")
                .collect();

            let move_file: bool = matches.is_present("move_file");

            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().ui_server_port;

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path)
                    .expect("failed to convert path into absolute path");

                let document =
                    import_document_from_file(&arhiv, document_type, &file_path, move_file)
                        .expect("failed to import file");

                print_document(&document, port);
            }
        }
        ("ui-server", _) => {
            start_ui_server().await;
        }
        ("ui-open", matches) => {
            let id: Id = matches.value_of("id").expect("id must be present").into();

            let port = Config::must_read().0.ui_server_port;

            let browser = matches
                .value_of("browser")
                .expect("either browser must be specified or $BROWSER env var must be set");

            log::info!("Opening document {} UI in {}", id, browser);

            process::Command::new(&browser)
                .arg(document_url(&id, port))
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to run browser {}", browser));
        }
        ("prime-server", matches) => {
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
        _ => unreachable!(),
    }
}

fn document_url(id: &Id, port: u16) -> String {
    format!("http://localhost:{}/documents/{}", port, id)
}

fn print_document(document: &Document, port: u16) {
    println!(
        "[{} {}] {}",
        document.document_type,
        document.id,
        document_url(&document.id, port)
    );
}

pub fn gen_completions_cli() {
    let manifest_dir = env!(
        "CARGO_MANIFEST_DIR",
        "CARGO_MANIFEST_DIR env variable is missing"
    );
    let outdir = format!("{}/completions", manifest_dir);

    let mut app = build_app();

    let bin_name = app
        .get_bin_name()
        .expect("failed to get bin name")
        .to_string();

    generate_to(Shell::Bash, &mut app, &bin_name, &outdir)
        .expect("failed to generate Bash completions");

    generate_to(Shell::Zsh, &mut app, &bin_name, &outdir)
        .expect("failed to generate Zsh completions");
}
