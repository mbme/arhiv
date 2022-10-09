use std::{process, sync::Arc};

use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};

use arhiv_core::{
    definitions::get_standard_schema,
    entities::{Document, DocumentData, Id},
    prime_server::start_prime_server,
    Arhiv, Config, ScraperOptions,
};
use arhiv_ui3::start_ui_server;
use rs_utils::{get_crate_version, into_absolute_path, log};

use crate::import::import_document_from_file;

#[derive(Parser, Debug)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true, disable_help_subcommand = true)]
#[command(name = "arhiv")]
struct CLIArgs {
    #[clap(subcommand)]
    command: CLICommand,

    /// Increases logging verbosity each use for up to 2 times
    #[clap(global= true, short, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    /// Initialize Arhiv instance on local machine
    Init {
        /// Arhiv id to use
        #[clap(long)]
        arhiv_id: String,

        /// Initialize prime instance
        #[clap(long)]
        prime: bool,
    },
    /// Sync changes
    Sync,
    /// Backup Arhiv data
    Backup {
        /// Directory to store backup. Will take precendence over config.backup_dir option.
        #[clap(long, value_hint = ValueHint::DirPath)]
        backup_dir: Option<String>,
    },
    /// Run arhiv UI server
    #[clap(name = "ui-server")]
    UIServer,
    /// Open UI for a document
    #[clap(name = "ui-open")]
    UIOpen {
        /// Document id to open
        #[arg()]
        id: Id,
        /// Open using provided browser or fall back to $BROWSER env variable
        #[clap(long, env = "BROWSER")]
        browser: String,
    },
    /// Run prime server
    #[clap(name = "prime-server")]
    PrimeServer {
        /// Listen on specific port
        #[clap(long, default_value = "23420")]
        port: u16,
    },
    /// Print current status
    Status,
    /// Print config
    Config {
        /// Prints config template
        #[clap(short, long)]
        template: bool,
    },
    /// Get document by id
    Get {
        /// Id of the document
        #[arg()]
        id: Id,
    },
    /// Add new document
    Add {
        /// One of known document types
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: String,
        /// Document subtype
        #[clap(long)]
        subtype: Option<String>,
        /// JSON object with document props
        #[arg()]
        data: String,
    },
    /// Scrape remote resource and create document
    Scrape {
        /// url to scrape
        #[arg(value_hint = ValueHint::Url)]
        url: String,
        /// Manual scraper mode
        #[clap(long, default_value = "false")]
        manual: bool,
        /// Emulate mobile mode
        #[clap(long, default_value = "false")]
        mobile: bool,
    },
    /// Import files and create documents. Will hard link or copy files to Arhiv.
    Import {
        /// One of known document types
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: String,
        /// Files to import
        #[arg(num_args = 1.., value_hint = ValueHint::FilePath)]
        file_paths: Vec<String>,
        /// Move file to arhiv
        #[arg(short, default_value = "false")]
        move_file: bool,
    },
    #[clap(name = "generate-completions", hide = true)]
    GenerateCompletions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[allow(clippy::too_many_lines)]
pub async fn arhiv_cli() {
    let args = CLIArgs::parse();

    match args.verbose {
        0 => log::setup_logger(),
        1 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    match args.command {
        CLICommand::Init { arhiv_id, prime } => {
            let config = Config::must_read().0;
            let schema = get_standard_schema();

            Arhiv::create(config, schema, &arhiv_id, prime).expect("must be able to create arhiv");
        }
        CLICommand::Status => {
            let status = Arhiv::must_open()
                .get_status()
                .expect("must be able to get status");

            println!("{}", status);
            // FIXME print number of unused temp attachments
        }
        CLICommand::Config { template } => {
            if template {
                print!("{}", include_str!("../resources/arhiv.json.template"));
                return;
            }

            let (config, path) = Config::must_read();
            println!("Arhiv config {}:", path);
            println!(
                "{}",
                serde_json::to_string_pretty(&config).expect("must be able to serialize config")
            );
        }
        CLICommand::Sync => {
            Arhiv::must_open().sync().await.expect("must sync");
        }
        CLICommand::Get { id } => {
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
        CLICommand::Add {
            document_type,
            subtype,
            data,
        } => {
            let data: DocumentData =
                serde_json::from_str(&data).expect("data must be a JSON object");

            let mut document =
                Document::new_with_data(&document_type, &subtype.unwrap_or_default(), data);

            let arhiv = Arhiv::must_open();

            let tx = arhiv.get_tx().expect("must open tx");

            tx.stage_document(&mut document)
                .expect("must be able to stage document");

            tx.commit().expect("must commit");

            let port = arhiv.get_config().ui_server_port;
            print_document(&document, port);
        }
        CLICommand::Scrape {
            url,
            manual,
            mobile,
        } => {
            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().ui_server_port;

            let documents = arhiv
                .scrape(
                    url,
                    ScraperOptions {
                        manual,
                        emulate_mobile: mobile,
                        debug: false,
                    },
                )
                .await
                .expect("failed to scrape");

            for document in documents {
                print_document(&document, port);
            }
        }
        CLICommand::Import {
            document_type,
            file_paths,
            move_file,
        } => {
            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().ui_server_port;

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path)
                    .expect("failed to convert path into absolute path");

                let document =
                    import_document_from_file(&arhiv, &document_type, &file_path, move_file)
                        .expect("failed to import file");

                print_document(&document, port);
            }
        }
        CLICommand::UIServer => {
            start_ui_server().await;
        }
        CLICommand::UIOpen { id, browser } => {
            log::info!("Opening document {} UI in {}", id, browser);

            let port = Config::must_read().0.ui_server_port;

            process::Command::new(&browser)
                .arg(document_url(&id, port))
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to run browser {}", browser));
        }
        CLICommand::PrimeServer { port } => {
            let arhiv = Arc::new(Arhiv::must_open());

            if !arhiv
                .get_status()
                .expect("must be able to get status")
                .db_status
                .is_prime
            {
                panic!("server must be started on prime instance");
            }

            let (join_handle, _, _) = start_prime_server(arhiv, port);

            join_handle.await.expect("must join");
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = Arhiv::must_open();

            let backup_dir = backup_dir.unwrap_or(arhiv.get_config().backup_dir.clone());

            arhiv.backup(&backup_dir).expect("must be able to backup");
        }
        CLICommand::GenerateCompletions { shell } => {
            let mut cmd = CLIArgs::command();

            let name = cmd.get_name().to_string();

            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
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
