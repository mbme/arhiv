use std::{env, process, time::Duration};

use anyhow::{Context, Result};
use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};
use dialoguer::{theme::ColorfulTheme, Input, Password};
use serde_json::Value;
use tokio::time::sleep;

use arhiv::{
    definitions::get_standard_schema, Arhiv, ArhivConfigExt, ArhivOptions, ArhivServer, ServerInfo,
};
use baza::{
    entities::{Document, DocumentData, DocumentLockKey, DocumentType, Id},
    KvsEntry, KvsKey, DEV_MODE,
};
use rs_utils::{get_crate_version, into_absolute_path, log, shutdown_signal, SecretString};

#[derive(Parser, Debug)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true, disable_help_subcommand = true)]
#[command(name = "arhiv")]
struct CLIArgs {
    #[clap(subcommand)]
    command: CLICommand,

    /// Increases logging verbosity each use for up to 3 times. Default level is WARN.
    /// Logs are written to stderr.
    #[clap(global= true, short, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum KVSCommand {
    /// List all entries
    List {
        /// Optional namespace of the entries
        namespace: Option<String>,
    },
    /// Get entry
    Get {
        /// Namespace of the entry
        namespace: String,
        /// Key of the entry
        key: String,
    },
    /// Set entry
    Set {
        /// Namespace of the entry
        namespace: String,
        /// Key of the entry
        key: String,
        /// Value of the entry, serialized as JSON. If not provided, the entry will be removed.
        value: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    /// Initialize Arhiv instance on local machine
    Init,
    /// Update Arhiv credentials
    #[clap(name = "update-credentials")]
    UpdateCredentials,
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
    Server {
        /// The port to listen on
        #[arg(long, env = "SERVER_PORT", default_value = "0")]
        port: u16,
    },
    /// Print server info, in JSON format
    #[clap(name = "server-info")]
    ServerInfo,
    /// Print current status
    Status,
    /// Print or update config
    Config {
        /// Auto-commit delay in seconds
        #[arg(long)]
        auto_commit_delay: Option<u64>,

        /// Auto-sync delay in seconds
        #[arg(long)]
        auto_sync_delay: Option<u64>,
    },
    /// Operations with KVS entries
    #[clap(subcommand)]
    Kvs(KVSCommand),
    /// List document locks
    Locks,
    /// Lock document
    Lock {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Reason why the document is being locked
        #[arg(default_value = "locked by CLI")]
        reason: String,
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
        /// JSON object with document props
        #[arg()]
        data: String,
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
        0 => log::setup_warn_logger(),
        1 => log::setup_logger(),
        2 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    handle_command(args.command).await.expect("command failed");
}

fn find_root_dir() -> Result<String> {
    let dir = if DEV_MODE {
        env::var("DEV_ARHIV_ROOT").context("env variable DEV_ARHIV_ROOT is missing")?
    } else {
        env::var("ARHIV_ROOT").context("env variable ARHIV_ROOT is missing")?
    };

    into_absolute_path(dir, false)
}

fn must_open_arhiv() -> Arhiv {
    let root_dir = find_root_dir().expect("must find root dir");

    Arhiv::open(root_dir, ArhivOptions::default()).expect("must be able to open arhiv")
}

async fn handle_command(command: CLICommand) -> Result<()> {
    match command {
        CLICommand::Init => {
            let root_dir = find_root_dir()?;

            let auth = prompt_credentials()?;

            Arhiv::create(root_dir, auth).context("must be able to create arhiv")?;
        }
        CLICommand::UpdateCredentials => {
            let arhiv = must_open_arhiv();

            println!("Please enter new credentials");
            let auth = prompt_credentials()?;

            arhiv.baza.update_credentials(auth)?;

            println!("Credentials updated");
        }
        CLICommand::Status => {
            let arhiv = must_open_arhiv();
            let status = arhiv.get_status()?;

            println!("{status}");
            // FIXME print number of unused temp assets
        }
        CLICommand::Config {
            auto_commit_delay,
            auto_sync_delay,
        } => {
            let arhiv = must_open_arhiv();

            let tx = arhiv.baza.get_tx()?;

            if let Some(auto_commit_delay) = auto_commit_delay {
                tx.set_auto_commit_delay(auto_commit_delay)?;
            }

            if let Some(auto_sync_delay) = auto_sync_delay {
                tx.set_auto_sync_delay(auto_sync_delay)?;
            }

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
        CLICommand::Kvs(command) => {
            let arhiv = must_open_arhiv();

            match command {
                KVSCommand::List { namespace } => {
                    let conn = arhiv.baza.get_connection()?;

                    let kvs_entries = conn.kvs_list(namespace.as_deref())?;

                    println!("{} entries", kvs_entries.len());
                    for KvsEntry(kvs_key, value) in kvs_entries {
                        println!("{kvs_key}: {value}");
                    }
                }
                KVSCommand::Get { namespace, key } => {
                    let conn = arhiv.baza.get_connection()?;

                    let kvs_key = KvsKey::new(namespace, key);
                    let value = conn.kvs_get_raw(&kvs_key)?;

                    if let Some(value) = value {
                        println!("{kvs_key}: {value}");
                    } else {
                        println!("{kvs_key} not found");
                    }
                }
                KVSCommand::Set {
                    namespace,
                    key,
                    value,
                } => {
                    let tx = arhiv.baza.get_tx()?;

                    let kvs_key = &KvsKey::new(namespace, key);
                    if let Some(value) = value {
                        let value: Value = serde_json::from_str(&value)
                            .context("Failed to parse provided value as JSON")?;

                        tx.kvs_set(kvs_key, &value)?;
                        println!("{kvs_key}: {value}");
                    } else {
                        tx.kvs_delete(kvs_key)?;
                        println!("Deleted {kvs_key}");
                    }

                    tx.commit()?;
                }
            };
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
                tx.unlock_document(&id, &DocumentLockKey::from_string(key))?;
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
            data,
        } => {
            let data: DocumentData =
                serde_json::from_str(&data).context("data must be a JSON object")?;

            let mut document = Document::new_with_data(DocumentType::new(document_type), data);

            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir.clone(),
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
            )?;

            let mut tx = arhiv.baza.get_tx()?;
            tx.stage_document(&mut document, None)?;

            tx.commit()?;

            let server_info = ServerInfo::collect(&root_dir)?;

            print_document(&document, &server_info);
        }
        CLICommand::Import {
            document_type,
            file_paths,
            move_file,
        } => {
            let root_dir = find_root_dir()?;
            let arhiv = Arhiv::open(
                root_dir.clone(),
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
            )?;
            let server_info = ServerInfo::collect(&root_dir)?;

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path, true)
                    .context("failed to convert path into absolute path")?;

                let document = arhiv
                    .import_document_from_file(&document_type, &file_path, move_file)
                    .context("failed to import file")?;

                print_document(&document, &server_info);
            }
        }
        CLICommand::UIOpen { id, browser } => {
            log::info!("Opening arhiv UI in {}", browser);

            let root_dir = find_root_dir()?;
            let server_info =
                ServerInfo::collect(&root_dir)?.context("Failed to collect server info")?;

            let url = id
                .map(|id| server_info.get_document_url(&id))
                .unwrap_or(server_info.ui_url);

            process::Command::new(&browser)
                .arg(url)
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to run browser {browser}"))
                .wait()
                .expect("Command wasn't running");
        }
        CLICommand::Server { port } => {
            let root_dir = find_root_dir()?;

            let server = ArhivServer::start(
                &root_dir,
                ArhivOptions {
                    auto_commit: true,
                    ..Default::default()
                },
                port,
            )
            .await?;

            if DEV_MODE {
                let server_info =
                    ServerInfo::collect(&root_dir)?.context("Failed to collect server info")?;
                log::info!("Dev server url: {}", server_info.ui_url_with_auth_token);
            }

            shutdown_signal().await;

            server.shutdown().await?;
        }
        CLICommand::ServerInfo => {
            let root_dir = find_root_dir()?;
            let server_info = ServerInfo::collect(&root_dir)?;

            let server_info =
                serde_json::to_string(&server_info).context("Failed to serialize ServerInfo")?;
            println!("{}", server_info);
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = must_open_arhiv();

            let backup_dir = into_absolute_path(backup_dir, true)?;

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

fn print_document(document: &Document, server_info: &Option<ServerInfo>) {
    let document_url = server_info
        .as_ref()
        .map(|server_info| server_info.get_document_url(&document.id))
        .unwrap_or_default();

    println!(
        "[{} {}] {}",
        document.document_type, document.id, document_url
    );
}

fn prompt_password(min_length: usize) -> Result<SecretString> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Password (min {min_length} symbols):"))
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.chars().count() >= min_length {
                Ok(())
            } else {
                Err("Password must be longer than {min_length}")
            }
        })
        .interact()
        .map(|value| value.into())
        .context("Failed to prompt password")
}

fn prompt_credentials() -> Result<SecretString> {
    prompt_password(Credentials::MIN_PASSWORD_LENGTH)
}
