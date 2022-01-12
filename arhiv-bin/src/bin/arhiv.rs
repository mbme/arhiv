#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    process::{self, Command, Stdio},
    sync::Arc,
};

use arhiv_bin::build_app;
use arhiv_core::{
    apply_migrations,
    definitions::{get_standard_schema, Attachment},
    entities::{Document, DocumentData, Id},
    prime_server::start_prime_server,
    Arhiv, Config,
};
use arhiv_scraper::scrape;
use arhiv_ui3::start_ui_server;
use rs_utils::{
    into_absolute_path,
    log::{self, setup_logger_with_level},
    EnvCapabilities,
};

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() {
    let matches = build_app().get_matches();

    setup_logger_with_level(matches.occurrences_of("verbose"));

    match matches.subcommand().expect("subcommand must be provided") {
        ("init", matches) => {
            let arhiv_id: String = matches
                .value_of("arhiv_id")
                .expect("arhiv_id must be present")
                .to_string();

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
                document_url(&document.id, arhiv.get_config().ui_server_port)
            );
        }
        ("attach", matches) => {
            let file_path: &str = matches
                .value_of("file_path")
                .expect("file_path must be present");

            let file_path =
                into_absolute_path(file_path).expect("failed to convert path to absolute");

            let move_file: bool = matches.is_present("move_file");

            let arhiv = Arhiv::must_open();

            let attachment = Attachment::create(&file_path, move_file, &arhiv)
                .expect("must be able to create attachment");

            println!("{}", &attachment.id);
        }
        ("scrape", matches) => {
            let url: &str = matches.value_of("url").expect("url must be present");

            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().ui_server_port;

            let capabilities = EnvCapabilities::must_check();
            let documents = scrape(&arhiv, &capabilities, url, false)
                .await
                .expect("failed to scrape");

            for document in documents {
                println!(
                    "{} {} {}",
                    document.document_type,
                    document.id,
                    document_url(&document.id, port)
                );
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

            Command::new(&browser)
                .arg(document_url(&id, port))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
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
