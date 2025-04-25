use std::{
    env,
    fs::{self, read_to_string},
    process,
};

use anyhow::{bail, Context, Result};
use clap::{
    builder::PossibleValuesParser, ArgAction, CommandFactory, Parser, Subcommand, ValueHint,
};
use clap_complete::{generate, Shell};
use dialoguer::{theme::ColorfulTheme, Password};

use arhiv::{definitions::get_standard_schema, Arhiv, ArhivOptions, ArhivServer};
use baza::{
    baza2::BazaManager,
    entities::{Document, DocumentData, DocumentLockKey, DocumentType, Id},
    DEV_MODE,
};
use rs_utils::{
    ensure_file_exists, file_exists, get_crate_version, image::generate_qrcode_svg,
    init_global_rayon_threadpool, into_absolute_path, log, shutdown_signal, SecretString,
};

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
    /// Save Arhiv password into system keyring
    Login,
    /// Erase Arhiv password from system keyring
    Logout,
    /// Change Arhiv password
    ChangePassword,
    /// Export Arhiv key file.
    ExportKey {
        /// Exported key file name.
        output_file: String,

        /// Encode key file as QR Code SVG image
        #[arg(long)]
        qrcode_svg: bool,
    },
    /// Verify if file is a valid Arhiv key file and can open Arhiv.
    VerifyKey {
        /// Key file to verify.
        #[arg(value_hint = ValueHint::FilePath)]
        key_file: String,
    },
    /// Import Arhiv key file and replace existing key file.
    ImportKey {
        /// Age key file to import.
        #[arg(value_hint = ValueHint::FilePath)]
        key_file: String,
        // TODO import from qrcode img as well
    },
    /// Backup Arhiv data
    Backup {
        /// Directory to store backup.
        #[arg(value_hint = ValueHint::DirPath)]
        backup_dir: String,
    },
    /// Run server
    Server {
        /// The port to listen on
        #[arg(long, env = "SERVER_PORT", default_value_t = ArhivServer::DEFAULT_PORT)]
        port: u16,

        /// Print server info as JSON. The line will start with @@SERVER_INFO:
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Open in $BROWSER
        #[arg(long, default_value_t = false)]
        browser: bool,
    },
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
        #[arg(short, default_value_t = false)]
        remove_original_file: bool,
    },
    #[clap(name = "generate-completions", hide = true)]
    GenerateCompletions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() {
    let args = CLIArgs::parse();

    match args.verbose {
        0 => log::setup_warn_logger(),
        1 => log::setup_logger(),
        2 => log::setup_debug_logger(),
        _ => log::setup_trace_logger(),
    };

    let worker_threads_count = Arhiv::optimal_number_of_worker_threads();
    log::debug!("Using {worker_threads_count} worker threads");

    init_global_rayon_threadpool(worker_threads_count)
        .expect("Failed to init global rayon thread pool");

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(worker_threads_count);
    builder.enable_all();
    let runtime = builder.build().expect("Failed to create tokio runtime");

    runtime
        .block_on(handle_command(args.command))
        .expect("Failed to handle command");
}

fn unlock_arhiv(arhiv: &Arhiv) {
    if !arhiv
        .baza
        .storage_exists()
        .expect("Failed to check if storage exists")
    {
        panic!("Arhiv not initialized");
    }

    if !arhiv
        .baza
        .key_exists()
        .expect("Failed to check if key exists")
    {
        panic!("Arhiv key is missing. First need to import key.");
    }

    match arhiv.unlock_using_keyring() {
        Ok(true) => return,
        Ok(false) => {
            log::debug!("Didn't find password in keyring");
        }
        Err(err) => {
            log::error!("Failed to use keyring: {err}");
        }
    }

    println!("Please enter password");
    let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)
        .expect("failed to prompt Arhiv password");

    arhiv.unlock(password).expect("Failed to unlock Arhiv")
}

async fn handle_command(command: CLICommand) -> Result<()> {
    match command {
        CLICommand::Init => {
            let arhiv = Arhiv::new_desktop();

            if arhiv.baza.storage_exists()? {
                bail!("Can't init: Arhiv storage already exists");
            }

            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, true)?;

            arhiv.create(password)?;

            println!("Done")
        }
        CLICommand::Login => {
            let arhiv = Arhiv::new_desktop();

            if !arhiv.baza.storage_exists()? {
                bail!("Can't login: Arhiv not initialized");
            }

            if !arhiv.baza.key_exists()? {
                bail!("Can't login: Arhiv key is missing. First need to import key.");
            }

            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            arhiv.unlock(password)?;

            println!("Saved password to keyring");
        }
        CLICommand::Logout => {
            let arhiv = Arhiv::new_desktop();
            arhiv.lock()?;

            println!("Erased password from keyring");
        }
        CLICommand::ChangePassword => {
            let arhiv = Arhiv::new_desktop();

            println!("Enter Arhiv password");
            let old_password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            // validate old password
            arhiv.unlock(old_password.clone())?;

            println!("Enter new Arhiv password");
            let new_password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, true)?;

            arhiv.change_password(old_password, new_password.clone())?;

            println!("Password changed");
        }
        CLICommand::ExportKey {
            output_file,
            qrcode_svg,
        } => {
            if file_exists(&output_file)? {
                bail!("Can't export key: file {output_file} already exists");
            }

            let arhiv = Arhiv::new_desktop();

            println!("Enter Arhiv password");
            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            // validate password
            arhiv.unlock(password.clone())?;

            println!("Enter new password for {output_file}");
            let new_password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, true)?;

            let key_data = arhiv.baza.export_key(password, new_password)?;

            if qrcode_svg {
                println!("Generating QR Code SVG image");
                let qrcode = generate_qrcode_svg(key_data.as_bytes())?;
                fs::write(&output_file, qrcode).context("Failed to write key into file")?;
            } else {
                fs::write(&output_file, key_data).context("Failed to write key into file")?;
            }

            println!("Exported key into {output_file}");
        }
        CLICommand::VerifyKey { key_file } => {
            ensure_file_exists(&key_file)?;

            let encrypted_key_data =
                read_to_string(&key_file).context("Failed to read key file")?;

            println!("Enter password for {key_file}");
            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            let arhiv = Arhiv::new_desktop();
            match arhiv.baza.verify_key(encrypted_key_data, password) {
                Ok(is_valid) => {
                    if is_valid {
                        println!("Key {key_file} can open Arhiv");
                    } else {
                        println!("Key {key_file} can't open Arhiv");
                    }
                }
                Err(err) => {
                    eprintln!(
                        "File {key_file} isn't a valid key file, or password is wrong: {err:?}"
                    );
                }
            }
        }
        CLICommand::ImportKey { key_file } => {
            ensure_file_exists(&key_file)?;

            let encrypted_key_data =
                read_to_string(&key_file).context("Failed to read key file")?;

            println!("Enter password for {key_file}");
            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            let arhiv = Arhiv::new_desktop();
            arhiv.baza.import_key(encrypted_key_data, password)?;

            println!("Imported key (and password) from {key_file}");
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
            let success = !baza.commit()?.is_empty();

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

            print_document(&document);
        }
        CLICommand::Import {
            document_type,
            file_paths,
            remove_original_file,
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            println!("Importing {} files", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path, true)
                    .context("failed to convert path into absolute path")?;

                let document = arhiv
                    .import_document_from_file(&document_type, &file_path, remove_original_file)
                    .context("failed to import file")?;

                print_document(&document);
            }
        }
        CLICommand::Server {
            port,
            json,
            browser,
        } => {
            let server = ArhivServer::start(ArhivOptions::new_desktop(), port).await?;
            let server_info = server.get_info();

            if json {
                eprintln!(
                    "@@SERVER_INFO: {}",
                    serde_json::to_string(server_info).expect("Failed to serialize ServerInfo")
                );
            }

            if browser {
                let browser =
                    env::var("BROWSER").context("Failed to read $BROWSER env variable")?;

                process::Command::new(&browser)
                    .arg(&server_info.ui_url_with_auth_token)
                    .stdout(process::Stdio::null())
                    .stderr(process::Stdio::null())
                    .spawn()
                    .unwrap_or_else(|_| panic!("Failed to run browser {browser}"))
                    .wait()
                    .expect("Command wasn't running");
            }

            if DEV_MODE {
                log::info!("Dev server url: {}", server_info.ui_url_with_auth_token);
            }

            shutdown_signal().await;

            server.shutdown().await?;
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

fn print_document(document: &Document) {
    println!("[{} {}]", document.document_type, document.id);
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
