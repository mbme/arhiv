use std::{env, process};

use anyhow::{Context, Result};
use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};

use arhiv::{definitions::get_standard_schema, Arhiv, ArhivConfigExt, ArhivOptions, ArhivServer};
use baza::entities::{Document, DocumentClass, DocumentData, Id};
use rs_utils::{get_crate_version, into_absolute_path, log};
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
        /// Directory to store backup.
        #[arg(value_hint = ValueHint::DirPath)]
        backup_dir: String,
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
    Server,
    /// Print current status
    Status,
    /// Print or update config
    Config {
        /// Server port
        #[arg(long)]
        server_port: Option<u16>,

        /// Auto-commit delay in seconds
        #[arg(long)]
        auto_commit_delay: Option<u64>,

        /// Auto-sync delay in seconds
        #[arg(long)]
        auto_sync_delay: Option<u64>,
    },
    /// List document locks
    Locks,
    /// Lock document
    Lock {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Reason why the document is being locked
        #[arg()]
        reason: Option<String>,
    },
    /// Unlock document
    Unlock {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Lock key to be checked before unlocking
        #[arg()]
        key: Option<String>,
    },
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

fn find_root_dir() -> Result<String> {
    if cfg!(feature = "production-mode") {
        env::var("ARHIV_ROOT").context("env variable ARHIV_ROOT is missing")
    } else {
        env::var("DEBUG_ARHIV_ROOT").context("env variable DEBUG_ARHIV_ROOT is missing")
    }
}

fn must_open_arhiv() -> Arhiv {
    let root_dir = find_root_dir().expect("must find root dir");

    Arhiv::open(root_dir, ArhivOptions::default()).expect("must be able to open arhiv")
}

async fn handle_command(command: CLICommand) -> Result<()> {
    match command {
        CLICommand::Init => {
            let root_dir = find_root_dir()?;
            Arhiv::open(
                root_dir,
                ArhivOptions {
                    create: true,
                    ..Default::default()
                },
            )
            .context("must be able to create arhiv")?;
        }
        CLICommand::Status => {
            let arhiv = must_open_arhiv();
            let status = arhiv.get_status().await?;

            println!("{status}");
            // FIXME print number of unused temp attachments
        }
        CLICommand::Config {
            server_port,
            auto_commit_delay,
            auto_sync_delay,
        } => {
            let arhiv = must_open_arhiv();

            let tx = arhiv.baza.get_tx()?;

            if let Some(server_port) = server_port {
                tx.set_server_port(server_port)?;
            }

            if let Some(auto_commit_delay) = auto_commit_delay {
                tx.set_auto_commit_delay(auto_commit_delay)?;
            }

            if let Some(auto_sync_delay) = auto_sync_delay {
                tx.set_auto_sync_delay(auto_sync_delay)?;
            }

            println!("      Server port: {}", tx.get_server_port()?);
            println!(
                "  Auto-sync delay: {} seconds",
                tx.get_auto_sync_delay()?.as_secs()
            );
            println!(
                "Auto-commit delay: {} seconds",
                tx.get_auto_commit_delay()?.as_secs()
            );

            tx.commit()?;
        }
        CLICommand::Locks => {
            let arhiv = must_open_arhiv();

            let locks = arhiv.baza.get_connection()?.list_document_locks()?;

            println!("Arhiv locks, {} entries", locks.len());
            for (id, lock) in locks {
                println!("  document {id}: {lock}");
            }
        }
        CLICommand::Lock { id, reason } => {
            let reason = reason.unwrap_or("locked by CLI".to_string());

            let arhiv = must_open_arhiv();

            let mut tx = arhiv.baza.get_tx()?;
            tx.lock_document(&id, reason)?;
            tx.commit()?;

            println!("Locked document {id}");
        }
        CLICommand::Unlock { id, key } => {
            let arhiv = must_open_arhiv();

            let mut tx = arhiv.baza.get_tx()?;
            if let Some(key) = key {
                tx.unlock_document(&id, &key)?;
            } else {
                println!("Lock key wasn't provided, unlocking without key check");
                tx.unlock_document_without_key(&id)?;
            }
            tx.commit()?;

            println!("Unlocked document {id}");
        }
        CLICommand::Commit => {
            let arhiv = must_open_arhiv();

            let mut tx = arhiv.baza.get_tx()?;
            let count = tx.commit_staged_documents()?;
            tx.commit()?;

            println!("Committed {count} staged documents");
        }
        CLICommand::Sync => {
            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    auto_commit: true,
                    discover_peers: true,
                    ..Default::default()
                },
            )?;
            arhiv.sync().await?;
        }
        CLICommand::Get { id } => {
            let arhiv = must_open_arhiv();

            let document = arhiv.baza.get_connection()?.get_document(&id)?;

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

            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
            )?;

            let mut tx = arhiv.baza.get_tx()?;
            tx.stage_document(&mut document, None)?;

            let port = tx.get_server_port()?;

            tx.commit()?;

            print_document(&document, port);
        }
        CLICommand::Scrape {
            url,
            manual,
            mobile,
        } => {
            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
            )?;
            let port = arhiv.baza.get_connection()?.get_server_port()?;

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
            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
            )?;
            let port = arhiv.baza.get_connection()?.get_server_port()?;

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

            let arhiv = must_open_arhiv();
            let port = arhiv.baza.get_connection()?.get_server_port()?;

            process::Command::new(&browser)
                .arg(get_document_url(&id.as_ref(), port))
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to run browser {browser}"));
        }
        CLICommand::Server => {
            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    auto_commit: true,
                    discover_peers: true,
                    mdns_server: true,
                    ..Default::default()
                },
            )?;

            let server = ArhivServer::start(arhiv)?;

            server.join().await?;
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = must_open_arhiv();

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
