use std::process;

use anyhow::{Context, Result};
use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};

use arhiv_ui::{
    build_ui_router, definitions::get_standard_schema, Arhiv, BazaConnectionExt, Config,
};
use baza::{
    entities::{Document, DocumentClass, DocumentData, Id},
    sync::build_rpc_router,
    KvsEntry, KvsKey,
};
use rs_utils::{get_crate_version, http_server::HttpServer, into_absolute_path, log};
use scraper::ScraperOptions;

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
    Init,
    /// One-shot sync without starting a server
    Sync,
    /// Backup Arhiv data
    Backup {
        /// Directory to store backup. Will take precendence over config.backup_dir option.
        #[clap(long, value_hint = ValueHint::DirPath)]
        backup_dir: Option<String>,
    },
    /// Open UI for a document
    #[clap(name = "ui-open")]
    UIOpen {
        /// Document id to open
        #[arg()]
        id: Option<Id>,
        /// Open using provided browser or fall back to $BROWSER env variable
        #[clap(long, env = "BROWSER")]
        browser: String,
    },
    /// Run server
    #[clap(name = "server")]
    Server,
    /// Print current status
    Status,
    /// Print config
    Config {
        /// Prints config template
        #[clap(short, long)]
        template: bool,
    },
    /// Print settings
    Settings,
    /// Commit pending changes
    Commit,
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

#[tokio::main]
async fn main() {
    let args = CLIArgs::parse();

    match args.verbose {
        0 => log::setup_logger(),
        1 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    handle_command(args.command).await.expect("command failed");
}

async fn handle_command(command: CLICommand) -> Result<()> {
    match command {
        CLICommand::Init => {
            Arhiv::create().context("must be able to create arhiv")?;
        }
        CLICommand::Status => {
            let arhiv = Arhiv::must_open();
            let conn = arhiv.baza.get_connection()?;
            let status = conn.get_status()?;

            println!("{status}");
            // FIXME print number of unused temp attachments
        }
        CLICommand::Config { template } => {
            if template {
                print!("{}", include_str!("../../resources/arhiv.json.template"));
                return Ok(());
            }

            let (config, path) = Config::must_read();
            println!("Arhiv config {path}:");
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        CLICommand::Settings => {
            let arhiv = Arhiv::must_open();

            let settings = arhiv.baza.get_connection()?.list_settings()?;

            println!("Arhiv settings, {} entries:", settings.len());
            for KvsEntry(KvsKey { namespace: _, key }, value) in settings {
                println!("  {:>25}: {value}", key);
            }
        }
        CLICommand::Commit => {
            let arhiv = Arhiv::must_open();

            let mut tx = arhiv.baza.get_tx()?;
            let count = tx.commit_staged_documents()?;
            tx.commit()?;

            println!("Committed {count} staged documents");
        }
        CLICommand::Sync => {
            let arhiv = Arhiv::must_open();

            arhiv.get_sync_service()?.sync().await?;
        }
        CLICommand::Get { id } => {
            let arhiv = Arhiv::must_open();

            let document = arhiv.baza.get_document(&id)?;

            if let Some(document) = document {
                serde_json::to_writer_pretty(std::io::stdout(), &document)?;
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
                serde_json::from_str(&data).context("data must be a JSON object")?;

            let mut document = Document::new_with_data(
                DocumentClass::new(document_type, subtype.unwrap_or_default()),
                data,
            );

            let arhiv = Arhiv::must_open();

            let tx = arhiv.baza.get_tx()?;
            tx.stage_document(&mut document)?;
            tx.commit()?;

            let port = arhiv.get_config().server_port;
            print_document(&document, port);
        }
        CLICommand::Scrape {
            url,
            manual,
            mobile,
        } => {
            let arhiv = Arhiv::must_open();
            let port = arhiv.get_config().server_port;

            let documents = arhiv
                .scrape(
                    url,
                    ScraperOptions {
                        manual,
                        emulate_mobile: mobile,
                        debug: false,
                        screenshot_file: None,
                    },
                )
                .await
                .context("failed to scrape")?;

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
            let port = arhiv.get_config().server_port;

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path, true)
                    .context("failed to convert path into absolute path")?;

                let document = arhiv
                    .import_document_from_file(&document_type, &file_path, move_file)
                    .context("failed to import file")?;

                print_document(&document, port);
            }
        }
        CLICommand::UIOpen { id, browser } => {
            log::info!("Opening arhiv UI in {}", browser);

            let port = Config::must_read().0.server_port;

            process::Command::new(&browser)
                .arg(get_document_url(&id.as_ref(), port))
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to run browser {browser}"));
        }
        CLICommand::Server => {
            let arhiv = Arhiv::must_open();

            let port = arhiv.get_config().server_port;

            let router = build_rpc_router(arhiv.baza.clone(), Some(build_ui_router(arhiv)));
            let server = HttpServer::start(router, port);

            server.join().await?;
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = Arhiv::must_open();

            let backup_dir = backup_dir.unwrap_or_else(|| arhiv.get_config().backup_dir.clone());

            arhiv
                .baza
                .backup(&backup_dir)
                .context("must be able to backup")?;
        }
        CLICommand::GenerateCompletions { shell } => {
            let mut cmd = CLIArgs::command();

            let name = cmd.get_name().to_string();

            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }

    Ok(())
}

fn get_document_url(id: &Option<&Id>, port: u16) -> String {
    let base = format!("http://localhost:{port}/ui");

    if let Some(id) = id {
        format!("{base}?id={id}")
    } else {
        base
    }
}

fn print_document(document: &Document, port: u16) {
    println!(
        "[{} {}] {}",
        document.class,
        document.id,
        get_document_url(&Some(&document.id), port)
    );
}
