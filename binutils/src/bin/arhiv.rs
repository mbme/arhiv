use std::process;

use anyhow::{bail, Context, Result};
use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};
use dialoguer::{theme::ColorfulTheme, Password};

use arhiv::{definitions::get_standard_schema, Arhiv, ArhivOptions, ArhivServer, ServerInfo};
use baza::{
    baza2::BazaManager,
    entities::{Document, DocumentData, DocumentLockKey, DocumentType, Id},
    DEV_MODE,
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
enum CLICommand {
    /// Initialize Arhiv instance on local machine
    Init,
    /// Change Arhiv password
    #[clap(name = "change-password")]
    ChangePassword,
    /// Backup Arhiv data
    Backup {
        /// Directory to store backup.
        #[arg(value_hint = ValueHint::DirPath)]
        backup_dir: String,
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
    /// Import files and create documents.
    Import {
        /// One of known document types
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: String,
        /// Files to import
        #[arg(num_args = 1.., value_hint = ValueHint::FilePath)]
        file_paths: Vec<String>,
        /// Remove original files
        #[arg(short, default_value = "false")]
        remove_original_file: bool,
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

fn unlock_arhiv(arhiv: &Arhiv) {
    if !arhiv
        .baza
        .storage_exists()
        .expect("Failed to check if storage exists")
    {
        panic!("Arhiv not initialized");
    }

    println!("Please enter password");
    let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)
        .expect("failed to prompt arhiv password");

    arhiv.baza.unlock(password).expect("Failed to unlock arhiv")
}

async fn handle_command(command: CLICommand) -> Result<()> {
    match command {
        CLICommand::Init => {
            let arhiv = Arhiv::new_desktop();

            if arhiv.baza.storage_exists()? {
                bail!("Can't init: arhiv storage already exists");
            }

            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, true)?;

            arhiv.baza.create(password)?;

            println!("Done")
        }
        CLICommand::ChangePassword => {
            let arhiv = Arhiv::new_desktop();

            println!("Enter password");
            let old_password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            println!("Enter new password");
            let new_password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, true)?;

            arhiv
                .baza
                .change_key_file_password(old_password, new_password)?;

            println!("Password changed");
        }
        CLICommand::Status => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let status = arhiv.get_status()?;

            println!("{status}");
        }
        CLICommand::Locks => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let baza = arhiv.baza.open()?;
            let locks = baza.list_document_locks();

            println!("Arhiv locks, {} entries", locks.len());
            for (id, lock) in locks {
                println!("  document {id}: {lock}");
            }
        }
        CLICommand::Lock { id, reason } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            baza.lock_document(&id, reason)?;
            baza.save_changes()?;

            println!("Locked document {id}");
        }
        CLICommand::Unlock { id, key } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            if let Some(key) = key {
                baza.unlock_document(&id, &DocumentLockKey::from_string(key))?;
            } else {
                println!("Lock key wasn't provided, unlocking without key check");
                baza.unlock_document_without_key(&id)?;
            }
            baza.save_changes()?;

            println!("Unlocked document {id}");
        }
        CLICommand::Commit => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            let success = baza.commit()?;

            if success {
                println!("Committed documents");
            }
        }
        CLICommand::Get { id } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let baza = arhiv.baza.open()?;
            let document = baza.get_document(&id);

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

            let document = Document::new_with_data(DocumentType::new(document_type), data);

            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            let document = baza.stage_document(document, &None)?.clone();

            baza.save_changes()?;

            let server_info = arhiv.collect_server_info()?;

            print_document(&document, &server_info);
        }
        CLICommand::Import {
            document_type,
            file_paths,
            remove_original_file,
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let server_info = arhiv.collect_server_info()?;

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path, true)
                    .context("failed to convert path into absolute path")?;

                let document = arhiv
                    .import_document_from_file(&document_type, &file_path, remove_original_file)
                    .context("failed to import file")?;

                print_document(&document, &server_info);
            }
        }
        CLICommand::Server { port } => {
            let server = ArhivServer::start(ArhivOptions::new_desktop(), port).await?;

            if DEV_MODE {
                let server_info = server
                    .arhiv
                    .collect_server_info()?
                    .context("Failed to collect server info")?;
                log::info!("Dev server url: {}", server_info.ui_url_with_auth_token);
            }

            shutdown_signal().await;

            server.shutdown().await?;
        }
        CLICommand::ServerInfo => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let server_info = arhiv.collect_server_info()?;

            let server_info =
                serde_json::to_string(&server_info).context("Failed to serialize ServerInfo")?;
            println!("{}", server_info);
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

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

fn prompt_password(min_length: usize, with_confirmation: bool) -> Result<SecretString> {
    let theme = ColorfulTheme::default();

    let mut input =
        Password::with_theme(&theme).with_prompt(format!("Password (min {min_length} symbols):"));

    if with_confirmation {
        input = input.with_confirmation("Repeat password", "Error: the passwords don't match.");
    }

    input = input.validate_with(|input: &String| -> Result<(), String> {
        if input.chars().count() >= min_length {
            Ok(())
        } else {
            Err(format!("Password must be longer than {min_length}"))
        }
    });

    input
        .interact()
        .map(|value| value.into())
        .context("Failed to prompt password")
}
