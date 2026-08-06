use std::{
    cmp::Ordering,
    env,
    fs::{self, read_to_string},
    io, process,
};

use anyhow::{Context, Result, bail, ensure};
use clap::{
    ArgAction, CommandFactory, Parser, Subcommand, ValueHint, builder::PossibleValuesParser,
};
use clap_complete::{Shell, generate};
use dialoguer::{Password, theme::ColorfulTheme};

use arhiv::{
    Arhiv, ArhivOptions, ArhivServer, CacheUnlockResult,
    definitions::{TRACK_TYPE, get_standard_schema},
    server::media::generate_qrcode_svg,
};
use baza::{
    Baza, BazaManager, DEV_MODE, DocumentExpert, DocumentHead, Filter, diff_document_data,
    entities::{Document, DocumentData, DocumentLockKey, DocumentType, Id, Revision},
    schema::DataSchema,
};
use baza_common::{
    SecretString, ensure_file_exists, file_exists, get_crate_version, init_global_rayon_threadpool,
    into_absolute_path, log, remove_file_if_exists, shutdown_signal,
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
    /// Save an Arhiv storage key into the system keyring
    Login,
    /// Erase the cached Arhiv storage key from the system keyring
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
    /// List recent documents
    List {
        /// Restrict results to a document type. Can be used more than once.
        #[arg(long = "type", value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_types: Vec<String>,
        /// Page number, starting at 0
        #[arg(long, default_value_t = 0)]
        page: u8,
        /// Show only conflicted documents
        #[arg(long, default_value_t = false)]
        conflicts: bool,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Search documents
    Search {
        /// Full-text search query
        #[arg(required = true, num_args = 1.., value_name = "QUERY")]
        query: Vec<String>,
        /// Restrict results to a document type. Can be used more than once.
        #[arg(long = "type", value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_types: Vec<String>,
        /// Page number, starting at 0
        #[arg(long, default_value_t = 0)]
        page: u8,
        /// Show only conflicted documents
        #[arg(long, default_value_t = false)]
        conflicts: bool,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// List conflicted documents
    Conflicts {
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Inspect document conflicts
    Conflict {
        #[command(subcommand)]
        command: ConflictCommand,
    },
    /// Discard staged changes
    Reset {
        /// Id of the document to reset
        #[arg()]
        id: Option<Id>,
        /// Reset every staged document
        #[arg(long, default_value_t = false)]
        all: bool,
        /// Lock key to be checked before resetting a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// List committed snapshots for a document
    History {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Work with committed document snapshots
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommand,
    },
    /// Stage a committed snapshot as the current document data
    Revert {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Snapshot revision, as printed by history
        #[arg()]
        rev: String,
        /// Lock key to be checked before updating a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Compare document data
    Diff {
        #[command(subcommand)]
        command: DiffCommand,
    },
    /// Get document by id
    Get {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print raw document head JSON
        #[arg(long, default_value_t = false)]
        json: bool,
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
    /// Replace an existing document's JSON data
    Update {
        /// Id of the document
        #[arg()]
        id: Id,
        /// JSON object with document props
        #[arg()]
        data: String,
        /// Lock key to be checked before updating a locked document
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Erase document by id
    Erase {
        /// Id of the document
        #[arg()]
        id: Id,
    },
    /// Print schema information
    Schema {
        /// Document type to describe. Prints all types when omitted.
        #[arg(value_parser = PossibleValuesParser::new(
                            get_standard_schema().get_document_types(),
                        ))]
        document_type: Option<String>,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Work with document collections
    Collection {
        #[command(subcommand)]
        command: CollectionCommand,
    },
    /// Work with encrypted assets
    Asset {
        #[command(subcommand)]
        command: AssetCommand,
    },
    /// Import files and create documents.
    Import {
        /// Document type to import
        #[arg(value_parser = PossibleValuesParser::new([TRACK_TYPE]))]
        document_type: String,
        /// Files to import
        #[arg(required = true, num_args = 1.., value_hint = ValueHint::FilePath)]
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

#[derive(Subcommand, Debug)]
enum CollectionCommand {
    /// List collections containing a document
    List {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// List members of a collection
    Members {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Add a document to a collection
    Add {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to add
        #[arg()]
        id: Id,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Remove a document from a collection
    Remove {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to remove
        #[arg()]
        id: Id,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
    /// Move a collection member to a new zero-based position
    Move {
        /// Id of the collection document
        #[arg()]
        collection_id: Id,
        /// Id of the document to move
        #[arg()]
        id: Id,
        /// New zero-based position
        #[arg(long)]
        to: usize,
        /// Lock key to be checked before updating a locked collection
        #[arg(long)]
        lock_key: Option<String>,
    },
}

enum CollectionUpdate {
    Add,
    Remove,
    Move { to: usize },
}

#[derive(Subcommand, Debug)]
enum ConflictCommand {
    /// Show conflict branches and the staged resolution for a document
    Show {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SnapshotCommand {
    /// Get a committed snapshot by document id and revision
    Get {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Snapshot revision, as printed by history
        #[arg()]
        rev: String,
        /// Print machine-readable JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DiffCommand {
    /// Diff a staged working copy against its latest committed original
    Staged {
        /// Id of the document
        #[arg()]
        id: Id,
    },
    /// Diff two committed snapshots of one document
    Snapshots {
        /// Id of the document
        #[arg()]
        id: Id,
        /// Left snapshot revision, as printed by history
        #[arg()]
        left_rev: String,
        /// Right snapshot revision, as printed by history
        #[arg()]
        right_rev: String,
    },
    /// Diff a conflict's staged resolution against each branch, or branch-to-branch
    Conflict {
        /// Id of the conflicted document
        #[arg()]
        id: Id,
    },
}

#[derive(Subcommand, Debug)]
enum AssetCommand {
    /// Create encrypted asset documents from local files
    Create {
        /// Files to store as encrypted assets
        #[arg(required = true, num_args = 1.., value_hint = ValueHint::FilePath)]
        file_paths: Vec<String>,
        /// Remove original files after assets are saved
        #[arg(short, default_value_t = false)]
        remove_original_file: bool,
    },
    /// Decrypt an asset into a local file
    Export {
        /// Id of the asset document
        #[arg()]
        id: Id,
        /// Output file path
        #[arg(value_hint = ValueHint::FilePath)]
        output_file: String,
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
        Ok(CacheUnlockResult::Unlocked) => return,
        Ok(CacheUnlockResult::NeedsPassword) => {
            log::debug!("No usable cached storage key");
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

            println!("Saved storage key to keyring");
        }
        CLICommand::Logout => {
            let arhiv = Arhiv::new_desktop();
            arhiv.lock()?;

            println!("Erased cached storage key from keyring");
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
            arhiv.import_key(encrypted_key_data, password)?;

            println!("Imported key and saved storage key to keyring from {key_file}");
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
        CLICommand::List {
            document_types,
            page,
            conflicts,
            json,
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let filter = build_filter(document_types, String::new(), page, conflicts);
            print_document_list(&arhiv, &filter, json)?;
        }
        CLICommand::Search {
            query,
            document_types,
            page,
            conflicts,
            json,
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let filter = build_filter(document_types, query.join(" "), page, conflicts);
            print_document_list(&arhiv, &filter, json)?;
        }
        CLICommand::Conflicts { json } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            print_conflicts(&arhiv, json)?;
        }
        CLICommand::Conflict { command } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            handle_conflict_command(&arhiv, command)?;
        }
        CLICommand::Reset { id, all, lock_key } => {
            ensure!(
                all ^ id.is_some(),
                "Provide one document id or --all, but not both"
            );
            ensure!(
                !all || lock_key.is_none(),
                "--lock-key can only be used with a document id"
            );

            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            if all {
                baza.reset_all_documents()?;
                baza.save_changes()?;

                println!("Reset all staged documents");
            } else {
                let id = id.expect("id is present");
                let lock_key = lock_key.map(DocumentLockKey::from_string);

                baza.reset_document(&id, &lock_key)?;
                baza.save_changes()?;

                println!("Reset document {id}");
            }
        }
        CLICommand::History { id, json } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            print_document_history(&arhiv, &id, json)?;
        }
        CLICommand::Snapshot {
            command: SnapshotCommand::Get { id, rev, json },
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let rev = parse_revision(&rev)?;
            let baza = arhiv.baza.open()?;
            let snapshot = baza.get_document_snapshot(&id, &rev)?;

            print_snapshot(&arhiv.baza.get_document_expert(), &snapshot, json)?;
        }
        CLICommand::Revert { id, rev, lock_key } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let rev = parse_revision(&rev)?;
            let lock_key = lock_key.map(DocumentLockKey::from_string);

            let mut baza = arhiv.baza.open_mut()?;
            let document = baza
                .revert_document_to_snapshot(&id, &rev, &lock_key)?
                .clone();
            baza.save_changes()?;

            print_document(&document);
            println!("Staged snapshot {} as document {id}", rev.to_safe_string());
        }
        CLICommand::Diff { command } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            handle_diff_command(&arhiv, command)?;
        }
        CLICommand::Get { id, json } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let baza = arhiv.baza.open()?;
            let head = baza.get_document(&id);

            if let Some(head) = head {
                if json {
                    serde_json::to_writer_pretty(std::io::stdout(), &head)?;
                } else {
                    print_document_details(&arhiv.baza.get_document_expert(), &baza, head)?;
                }
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
        CLICommand::Update { id, data, lock_key } => {
            let data: DocumentData =
                serde_json::from_str(&data).context("data must be a JSON object")?;
            let lock_key = lock_key.map(DocumentLockKey::from_string);

            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut document = {
                let baza = arhiv.baza.open()?;
                baza.must_get_document(&id)?.clone()
            };
            document.data = data;

            let mut baza = arhiv.baza.open_mut()?;
            let document = baza.stage_document(document, &lock_key)?.clone();
            baza.save_changes()?;

            print_document(&document);
        }
        CLICommand::Erase { id } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let mut baza = arhiv.baza.open_mut()?;
            baza.erase_document(&id)?;
            baza.save_changes()?;

            println!("Erased document {id}");
        }
        CLICommand::Schema {
            document_type,
            json,
        } => {
            let arhiv = Arhiv::new_desktop();

            print_schema(arhiv.baza.get_schema(), document_type, json)?;
        }
        CLICommand::Collection { command } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            handle_collection_command(&arhiv, command)?;
        }
        CLICommand::Asset {
            command:
                AssetCommand::Create {
                    file_paths,
                    remove_original_file,
                },
        } => {
            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            println!("Creating {} assets", file_paths.len());

            for file_path in file_paths {
                let file_path = into_absolute_path(file_path, true)
                    .context("failed to convert path into absolute path")?;

                let asset = {
                    let mut baza = arhiv.baza.open_mut()?;
                    let asset = baza.create_asset(&file_path)?;
                    baza.save_changes()?;
                    asset
                };

                if remove_original_file {
                    remove_file_if_exists(&file_path)?;
                }

                let document = asset.into_document()?;
                print_document(&document);
            }
        }
        CLICommand::Asset {
            command: AssetCommand::Export { id, output_file },
        } => {
            if file_exists(&output_file)? {
                bail!("Can't export asset: file {output_file} already exists");
            }

            let arhiv = Arhiv::new_desktop();
            unlock_arhiv(&arhiv);

            let baza = arhiv.baza.open()?;
            let mut asset_data = baza.get_asset_data(&id)?;
            let mut output =
                fs::File::create(&output_file).context("Failed to create output file")?;
            io::copy(&mut asset_data, &mut output).context("Failed to write asset data")?;

            println!("Exported asset {id} into {output_file}");
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

                log::info!("Browser URL: {}", server_info.browser_url);
                launch_browser(&browser, &server_info.browser_url)?;
            } else if DEV_MODE {
                log::info!("Dev server url: {}", server_info.browser_url);
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

fn build_filter(
    document_types: Vec<String>,
    query: String,
    page: u8,
    only_conflicts: bool,
) -> Filter {
    Filter {
        document_types: document_types.into_iter().map(DocumentType::new).collect(),
        query,
        page,
        only_conflicts,
    }
}

fn handle_collection_command(arhiv: &Arhiv, command: CollectionCommand) -> Result<()> {
    match command {
        CollectionCommand::List { id, json } => {
            let document_expert = arhiv.baza.get_document_expert();
            let baza = arhiv.baza.open()?;
            get_document_head(&baza, &id)?;

            let mut collection_ids = baza
                .find_document_collections(&id)
                .into_iter()
                .collect::<Vec<_>>();
            collection_ids.sort_by_key(|id| id.to_string());

            print_documents_by_ids(
                &document_expert,
                &baza,
                &collection_ids,
                json,
                "Collections",
                "No collections found",
            )?;
        }
        CollectionCommand::Members {
            collection_id,
            json,
        } => {
            let document_expert = arhiv.baza.get_document_expert();
            let baza = arhiv.baza.open()?;
            let collection = get_document_head(&baza, &collection_id)?.get_single_document();
            let member_ids = document_expert.collection_member_ids(collection)?;

            print_documents_by_ids(
                &document_expert,
                &baza,
                &member_ids,
                json,
                "Members",
                "No members found",
            )?;
        }
        CollectionCommand::Add {
            collection_id,
            id,
            lock_key,
        } => {
            update_collection(arhiv, &collection_id, &id, lock_key, CollectionUpdate::Add)?;

            println!("Added document {id} to collection {collection_id}");
        }
        CollectionCommand::Remove {
            collection_id,
            id,
            lock_key,
        } => {
            update_collection(
                arhiv,
                &collection_id,
                &id,
                lock_key,
                CollectionUpdate::Remove,
            )?;

            println!("Removed document {id} from collection {collection_id}");
        }
        CollectionCommand::Move {
            collection_id,
            id,
            to,
            lock_key,
        } => {
            update_collection(
                arhiv,
                &collection_id,
                &id,
                lock_key,
                CollectionUpdate::Move { to },
            )?;

            println!("Moved document {id} in collection {collection_id} to position {to}");
        }
    }

    Ok(())
}

fn update_collection(
    arhiv: &Arhiv,
    collection_id: &Id,
    id: &Id,
    lock_key: Option<String>,
    update: CollectionUpdate,
) -> Result<()> {
    let lock_key = lock_key.map(DocumentLockKey::from_string);
    let document_expert = arhiv.baza.get_document_expert();
    let mut baza = arhiv.baza.open_mut()?;
    let mut collection = baza.must_get_document(collection_id)?.clone();

    match update {
        CollectionUpdate::Add => {
            let document = baza.must_get_document(id)?.clone();
            document_expert.add_document_to_collection(&document, &mut collection)?;
        }
        CollectionUpdate::Remove => {
            document_expert.remove_member_from_collection(&mut collection, id)?;
        }
        CollectionUpdate::Move { to } => {
            document_expert.reorder_collection_member(&mut collection, id, to)?;
        }
    }

    baza.stage_document(collection, &lock_key)?;
    baza.save_changes()?;

    Ok(())
}

fn handle_conflict_command(arhiv: &Arhiv, command: ConflictCommand) -> Result<()> {
    match command {
        ConflictCommand::Show { id, json } => {
            let baza = arhiv.baza.open()?;
            let head = get_document_head(&baza, &id)?;
            ensure!(head.is_conflict(), "Document {id} is not conflicted");

            print_conflict_details(&arhiv.baza.get_document_expert(), head, json)?;
        }
    }

    Ok(())
}

fn handle_diff_command(arhiv: &Arhiv, command: DiffCommand) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;

    match command {
        DiffCommand::Staged { id } => {
            let head = get_document_head(&baza, &id)?;
            ensure!(head.is_staged(), "Document {id} has no staged changes");
            ensure!(
                !head.is_new_document(),
                "Document {id} is new; no committed original to diff"
            );

            let original = latest_original_snapshot(head);
            let staged = head
                .get_staged_document()
                .expect("staged document is present");

            print_document_data_diff(&document_expert, "original", original, "staged", staged)?;
        }
        DiffCommand::Snapshots {
            id,
            left_rev,
            right_rev,
        } => {
            let left_rev = parse_revision(&left_rev)?;
            let right_rev = parse_revision(&right_rev)?;
            let left = baza.get_document_snapshot(&id, &left_rev)?;
            let right = baza.get_document_snapshot(&id, &right_rev)?;

            print_document_data_diff(
                &document_expert,
                "left snapshot",
                &left,
                "right snapshot",
                &right,
            )?;
        }
        DiffCommand::Conflict { id } => {
            let head = get_document_head(&baza, &id)?;
            ensure!(head.is_conflict(), "Document {id} is not conflicted");

            let branches = sorted_original_snapshots(head);
            if let Some(staged) = head.get_staged_document() {
                for (index, branch) in branches.iter().enumerate() {
                    if index > 0 {
                        println!();
                    }

                    print_document_data_diff(
                        &document_expert,
                        &format!("branch {}", index + 1),
                        branch,
                        "staged resolution",
                        staged,
                    )?;
                }
            } else {
                let first = branches
                    .first()
                    .expect("conflict must have at least two branches");

                for (index, branch) in branches.iter().enumerate().skip(1) {
                    if index > 1 {
                        println!();
                    }

                    print_document_data_diff(
                        &document_expert,
                        "branch 1",
                        first,
                        &format!("branch {}", index + 1),
                        branch,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn print_conflicts(arhiv: &Arhiv, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    let mut conflicts = baza.iter_conflicts().collect::<Vec<_>>();
    conflicts.sort_by_key(|head| head.get_id().to_string());

    if json_output {
        let documents = conflicts
            .iter()
            .map(|head| conflict_summary_json(&document_expert, head))
            .collect::<Result<Vec<_>>>()?;
        let total = documents.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if conflicts.is_empty() {
        println!("No conflicts found");
        return Ok(());
    }

    println!("Conflicts: {}", conflicts.len());
    for head in conflicts {
        print_conflict_row(&document_expert, head)?;
    }

    Ok(())
}

fn print_document_history(arhiv: &Arhiv, id: &Id, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    get_document_head(&baza, id)?;
    let snapshots = baza.list_document_snapshots(id)?;

    if json_output {
        let snapshots = snapshots
            .iter()
            .map(|document| snapshot_json(&document_expert, document))
            .collect::<Result<Vec<_>>>()?;
        let total = snapshots.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "id": id,
                "snapshots": snapshots,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if snapshots.is_empty() {
        println!("No committed snapshots found for document {id}");
        return Ok(());
    }

    println!("History for document {id}: {} snapshots", snapshots.len());
    for document in &snapshots {
        print_snapshot_row(&document_expert, document)?;
    }

    Ok(())
}

fn print_conflict_details(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
    json_output: bool,
) -> Result<()> {
    let branches = sorted_original_snapshots(head);

    if json_output {
        let branches = branches
            .iter()
            .map(|document| snapshot_json(document_expert, document))
            .collect::<Result<Vec<_>>>()?;
        let staged = head
            .get_staged_document()
            .map(|document| snapshot_json(document_expert, document))
            .transpose()?;

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "id": head.get_id(),
                "isResolved": head.is_resolved_conflict(),
                "staged": staged,
                "branches": branches,
                "branchesCount": branches.len(),
                "snapshotsCount": head.get_snapshots_count(),
            }),
        )?;
        return Ok(());
    }

    println!("Conflict for document {}", head.get_id());
    println!("Resolved: {}", head.is_resolved_conflict());
    println!("Branches: {}", branches.len());
    println!("Snapshots: {}", head.get_snapshots_count());

    if let Some(staged) = head.get_staged_document() {
        println!();
        print_snapshot_block(document_expert, "Staged resolution", staged)?;
    }

    for (index, document) in branches.iter().enumerate() {
        println!();
        print_snapshot_block(document_expert, &format!("Branch {}", index + 1), document)?;
    }

    Ok(())
}

fn print_snapshot(
    document_expert: &DocumentExpert<'_>,
    document: &Document,
    json_output: bool,
) -> Result<()> {
    if json_output {
        serde_json::to_writer_pretty(
            std::io::stdout(),
            &snapshot_json(document_expert, document)?,
        )?;
    } else {
        print_snapshot_block(document_expert, "Snapshot", document)?;
    }

    Ok(())
}

fn print_document_data_diff(
    document_expert: &DocumentExpert<'_>,
    left_role: &str,
    left: &Document,
    right_role: &str,
    right: &Document,
) -> Result<()> {
    let left_label = document_diff_label(document_expert, left_role, left)?;
    let right_label = document_diff_label(document_expert, right_role, right)?;
    let diff = diff_document_data(&left_label, left, &right_label, right)?;

    if diff.has_changes {
        print!("{}", diff.unified_diff);
    } else {
        println!("No data differences between {left_role} and {right_role}");
    }

    Ok(())
}

fn document_diff_label(
    document_expert: &DocumentExpert<'_>,
    role: &str,
    document: &Document,
) -> Result<String> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(format!(
        "{}: {} {} rev {} updated {} title {}",
        role,
        document.id,
        document.document_type,
        document.rev.to_safe_string(),
        document.updated_at.default_date_time_format(),
        single_line(&title)
    ))
}

fn print_snapshot_block(
    document_expert: &DocumentExpert<'_>,
    label: &str,
    document: &Document,
) -> Result<()> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;
    let data = serde_json::to_string_pretty(&document.data)?;

    println!("{label}");
    println!("  Id: {}", document.id);
    println!("  Rev: {}", document.rev.to_safe_string());
    println!("  Type: {}", document.document_type);
    println!("  Title: {}", title);
    println!(
        "  Updated: {}",
        document.updated_at.default_date_time_format()
    );
    println!("  Data:\n{data}");

    Ok(())
}

fn print_snapshot_row(document_expert: &DocumentExpert<'_>, document: &Document) -> Result<()> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {}  {}",
        document.rev.to_safe_string(),
        document.document_type,
        document.updated_at.default_date_time_format(),
        document.id,
        single_line(&title),
    );

    Ok(())
}

fn print_conflict_row(document_expert: &DocumentExpert<'_>, head: &DocumentHead) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {} [branches: {}, staged: {}]",
        document.id,
        document.document_type,
        document.updated_at.default_date_time_format(),
        single_line(&title),
        head.iter_original_snapshots().count(),
        head.is_staged(),
    );

    Ok(())
}

fn conflict_summary_json(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
) -> Result<serde_json::Value> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "isResolved": head.is_resolved_conflict(),
        "hasStaged": head.is_staged(),
        "branchesCount": head.iter_original_snapshots().count(),
        "snapshotsCount": head.get_snapshots_count(),
    }))
}

fn snapshot_json(
    document_expert: &DocumentExpert<'_>,
    document: &Document,
) -> Result<serde_json::Value> {
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "rev": &document.rev,
        "revSafe": document.rev.to_safe_string(),
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "data": &document.data,
        "isStaged": document.is_staged(),
    }))
}

fn representative_document(head: &DocumentHead) -> &Document {
    head.get_staged_document()
        .unwrap_or_else(|| latest_original_snapshot(head))
}

fn latest_original_snapshot(head: &DocumentHead) -> &Document {
    head.iter_original_snapshots()
        .max_by(|a, b| compare_documents_by_history(a, b))
        .expect("document head must have an original snapshot")
}

fn sorted_original_snapshots(head: &DocumentHead) -> Vec<&Document> {
    let mut snapshots = head.iter_original_snapshots().collect::<Vec<_>>();
    snapshots.sort_by(|a, b| compare_documents_by_history(a, b));

    snapshots
}

fn compare_documents_by_history(a: &Document, b: &Document) -> Ordering {
    a.updated_at
        .cmp(&b.updated_at)
        .then_with(|| a.rev.cmp(&b.rev))
}

fn parse_revision(value: &str) -> Result<Revision> {
    Revision::from_safe_string(value).with_context(|| format!("Failed to parse revision '{value}'"))
}

fn print_documents_by_ids(
    document_expert: &DocumentExpert<'_>,
    baza: &Baza,
    ids: &[Id],
    json_output: bool,
    label: &str,
    empty_message: &str,
) -> Result<()> {
    if json_output {
        let documents = ids
            .iter()
            .map(|id| {
                let head = get_document_head(baza, id)?;
                document_summary_json(document_expert, head)
            })
            .collect::<Result<Vec<_>>>()?;
        let total = documents.len();

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "total": total,
            }),
        )?;
        return Ok(());
    }

    if ids.is_empty() {
        println!("{empty_message}");
        return Ok(());
    }

    println!("{label}: {}", ids.len());

    for id in ids {
        let head = get_document_head(baza, id)?;
        print_document_row(document_expert, head)?;
    }

    Ok(())
}

fn get_document_head<'b>(baza: &'b Baza, id: &Id) -> Result<&'b DocumentHead> {
    baza.get_document(id)
        .with_context(|| format!("Can't find document {id}"))
}

fn print_document_list(arhiv: &Arhiv, filter: &Filter, json_output: bool) -> Result<()> {
    let document_expert = arhiv.baza.get_document_expert();
    let baza = arhiv.baza.open()?;
    let page = baza.list_documents(filter)?;

    if json_output {
        let documents = page
            .items
            .into_iter()
            .map(|head| document_summary_json(&document_expert, head))
            .collect::<Result<Vec<_>>>()?;

        serde_json::to_writer_pretty(
            std::io::stdout(),
            &serde_json::json!({
                "documents": documents,
                "hasMore": page.has_more,
                "total": page.total,
            }),
        )?;
        return Ok(());
    }

    if page.total == 0 {
        println!("No documents found");
        return Ok(());
    }

    println!(
        "Documents: {} total, showing {}{}",
        page.total,
        page.items.len(),
        if page.has_more {
            ", more available"
        } else {
            ""
        }
    );

    for head in page.items {
        print_document_row(&document_expert, head)?;
    }

    Ok(())
}

fn document_summary_json(
    document_expert: &DocumentExpert<'_>,
    head: &DocumentHead,
) -> Result<serde_json::Value> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    Ok(serde_json::json!({
        "id": &document.id,
        "documentType": &document.document_type,
        "title": title,
        "updatedAt": document.updated_at,
        "data": &document.data,
        "hasConflict": head.is_conflict(),
        "isStaged": head.is_staged(),
        "snapshotsCount": head.get_snapshots_count(),
    }))
}

fn print_document_row(document_expert: &DocumentExpert<'_>, head: &DocumentHead) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;

    println!(
        "{}  {:<12}  {}  {}{}",
        document.id,
        document.document_type,
        document.updated_at.default_date_time_format(),
        single_line(&title),
        status_flags(head)
    );

    Ok(())
}

fn print_document_details(
    document_expert: &DocumentExpert<'_>,
    baza: &Baza,
    head: &DocumentHead,
) -> Result<()> {
    let document = representative_document(head);
    let title = document_expert.get_title(&document.document_type, &document.data)?;
    let refs = document_expert.extract_refs(&document.document_type, &document.data)?;
    let data = serde_json::to_string_pretty(&document.data)?;

    println!("Id: {}", document.id);
    println!("Type: {}", document.document_type);
    println!("Title: {}", title);
    println!(
        "Updated: {}",
        document.updated_at.default_date_time_format()
    );
    println!("Staged: {}", head.is_staged());
    println!("Conflict: {}", head.is_conflict());
    println!("Snapshots: {}", head.get_snapshots_count());
    println!("Refs: {}", format_ids(refs.get_all_document_refs()));
    println!(
        "Backrefs: {}",
        format_ids(baza.find_document_backrefs(&document.id))
    );
    println!(
        "Collections: {}",
        format_ids(baza.find_document_collections(&document.id))
    );
    println!("Data:\n{data}");

    Ok(())
}

fn print_schema(
    schema: &DataSchema,
    document_type: Option<String>,
    json_output: bool,
) -> Result<()> {
    if let Some(document_type) = document_type {
        let document_type = DocumentType::new(document_type);
        let description = schema.get_data_description(&document_type)?;

        if json_output {
            serde_json::to_writer_pretty(std::io::stdout(), description)?;
            return Ok(());
        }

        println!("Document type: {}", description.document_type);
        println!("Title format: {}", description.title_format);
        println!("Fields:");
        for field in &description.fields {
            println!(
                "  {}: {:?}{}{}",
                field.name,
                field.field_type,
                if field.mandatory { ", mandatory" } else { "" },
                if field.readonly { ", readonly" } else { "" }
            );
        }

        return Ok(());
    }

    if json_output {
        serde_json::to_writer_pretty(std::io::stdout(), schema)?;
        return Ok(());
    }

    println!("Document types:");
    for document_type in schema.get_document_types() {
        println!("  {document_type}");
    }

    Ok(())
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn status_flags(head: &DocumentHead) -> String {
    let mut flags = Vec::new();

    if head.is_staged() {
        flags.push("staged");
    }
    if head.is_conflict() {
        flags.push("conflict");
    }

    if flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", flags.join(", "))
    }
}

fn format_ids(ids: impl IntoIterator<Item = Id>) -> String {
    let mut ids = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
    ids.sort();

    if ids.is_empty() {
        "-".to_string()
    } else {
        ids.join(", ")
    }
}

fn launch_browser(browser: &str, browser_url: &str) -> Result<()> {
    let mut command = process::Command::new(browser);
    command
        .arg(browser_url)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        command.process_group(0);
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to run browser {browser}"))?;

    let _ = std::thread::spawn(move || {
        if let Err(err) = child.wait() {
            log::warn!("Failed to wait for browser process: {err}");
        }
    });

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
