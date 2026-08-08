use std::{
    fs::{self, read_to_string},
    io, process,
};

use anyhow::{Context, Result, bail, ensure};
use clap::CommandFactory;
use clap_complete::generate;

use arhiv::{Arhiv, server::media::generate_qrcode_svg};
use baza::{
    BazaManager, Filter, RestoreCheckReport, RestoreOptions,
    entities::{Document, DocumentData, DocumentLockKey, DocumentType, Id, Revision},
};
use baza_common::{ensure_file_exists, file_exists, into_absolute_path, remove_file_if_exists};

use crate::{
    cli::{
        AssetCommand, CLIArgs, CLICommand, CollectionCommand, ConflictCommand, DiffCommand,
        RestoreCommand, SnapshotCommand,
    },
    output::{
        get_document_head, latest_original_snapshot, print_conflict_details, print_conflicts,
        print_document, print_document_data_diff, print_document_details, print_document_history,
        print_document_list, print_documents_by_ids, print_schema, print_snapshot,
        sorted_original_snapshots,
    },
    server::handle_server_command,
    session::{prompt_password, unlocked_desktop_arhiv},
};

enum CollectionUpdate {
    Add,
    Remove,
    Move { to: usize },
}

pub(crate) async fn handle_command(command: CLICommand) -> Result<()> {
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
            let arhiv = unlocked_desktop_arhiv()?;

            let status = arhiv.get_status()?;

            println!("{status}");
        }
        CLICommand::Locks => {
            let arhiv = unlocked_desktop_arhiv()?;

            let baza = arhiv.baza.open()?;
            let locks = baza.list_document_locks();

            println!("Arhiv locks, {} entries", locks.len());
            for (id, lock) in locks {
                println!("  document {id}: {lock}");
            }
        }
        CLICommand::Lock { id, reason } => {
            let arhiv = unlocked_desktop_arhiv()?;

            let mut baza = arhiv.baza.open_mut()?;
            baza.lock_document(&id, reason)?;
            baza.save_changes()?;

            println!("Locked document {id}");
        }
        CLICommand::Unlock { id, key } => {
            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

            let filter = build_filter(document_types, query.join(" "), page, conflicts);
            print_document_list(&arhiv, &filter, json)?;
        }
        CLICommand::Conflicts { json } => {
            let arhiv = unlocked_desktop_arhiv()?;

            print_conflicts(&arhiv, json)?;
        }
        CLICommand::Conflict { command } => {
            let arhiv = unlocked_desktop_arhiv()?;

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

            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

            print_document_history(&arhiv, &id, json)?;
        }
        CLICommand::Snapshot {
            command: SnapshotCommand::Get { id, rev, json },
        } => {
            let arhiv = unlocked_desktop_arhiv()?;

            let rev = parse_revision(&rev)?;
            let baza = arhiv.baza.open()?;
            let snapshot = baza.get_document_snapshot(&id, &rev)?;

            print_snapshot(&arhiv.baza.get_document_expert(), &snapshot, json)?;
        }
        CLICommand::Revert { id, rev, lock_key } => {
            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

            handle_diff_command(&arhiv, command)?;
        }
        CLICommand::Get { id, json } => {
            let arhiv = unlocked_desktop_arhiv()?;

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

            let arhiv = unlocked_desktop_arhiv()?;

            let mut baza = arhiv.baza.open_mut()?;
            let document = baza.stage_document(document, &None)?.clone();

            baza.save_changes()?;

            print_document(&document);
        }
        CLICommand::Update { id, data, lock_key } => {
            let data: DocumentData =
                serde_json::from_str(&data).context("data must be a JSON object")?;
            let lock_key = lock_key.map(DocumentLockKey::from_string);

            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

            handle_collection_command(&arhiv, command)?;
        }
        CLICommand::Asset {
            command:
                AssetCommand::Create {
                    file_paths,
                    remove_original_file,
                },
        } => {
            let arhiv = unlocked_desktop_arhiv()?;

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

            let arhiv = unlocked_desktop_arhiv()?;

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
            let arhiv = unlocked_desktop_arhiv()?;

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
            handle_server_command(port, json, browser).await?;
        }
        CLICommand::Backup { backup_dir } => {
            let arhiv = unlocked_desktop_arhiv()?;

            arhiv
                .baza
                .backup(&backup_dir)
                .context("must be able to backup")?;
        }
        CLICommand::Restore { command } => {
            handle_restore_command(command)?;
        }
        CLICommand::GenerateCompletions { shell } => {
            let mut cmd = CLIArgs::command();

            let name = cmd.get_name().to_string();

            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }

    Ok(())
}

fn handle_restore_command(command: RestoreCommand) -> Result<()> {
    match command {
        RestoreCommand::Check {
            manifest_path,
            allow_missing_blobs,
            deep,
        } => {
            ensure_file_exists(&manifest_path)?;
            println!("Enter password for backup key");
            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            let arhiv = Arhiv::new_desktop();
            let report = arhiv.baza.restore_check(
                &manifest_path,
                password,
                RestoreOptions {
                    allow_missing_blobs,
                    deep,
                    allow_rollback: false,
                },
            )?;

            print_restore_check_report(&report, false);
        }
        RestoreCommand::Apply {
            manifest_path,
            allow_missing_blobs,
            deep,
            allow_rollback,
        } => {
            ensure_file_exists(&manifest_path)?;
            let arhiv = unlocked_desktop_arhiv()?;

            println!("Enter password for backup key");
            let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

            let report = arhiv.baza.restore_apply(
                &manifest_path,
                password,
                RestoreOptions {
                    allow_missing_blobs,
                    deep,
                    allow_rollback,
                },
            )?;
            arhiv
                .lock()
                .context("Failed to clear cached storage key after restore")?;

            print_restore_check_report(&report, true);
        }
    }

    Ok(())
}

fn print_restore_check_report(report: &RestoreCheckReport, applied: bool) {
    if applied {
        println!("Restored backup {}", report.timestamp);
    } else {
        println!("Backup {} passed restore check", report.timestamp);
    }
    println!("  manifest: {}", report.manifest_path);
    println!("  referenced blobs: {}", report.referenced_blobs);
    println!("  missing blobs: {}", report.missing_blobs.len());
    println!("  verified artifacts: {}", report.verified_artifacts);
    if report.deep_verified_blobs > 0 {
        println!("  deep verified blobs: {}", report.deep_verified_blobs);
    }
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

fn parse_revision(value: &str) -> Result<Revision> {
    Revision::from_safe_string(value).with_context(|| format!("Failed to parse revision '{value}'"))
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
