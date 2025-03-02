use std::fs::{remove_dir, remove_file};

use anyhow::{ensure, Context, Result};
use arhiv::Arhiv;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Password};
use rusqlite::Row;
use serde_json::Value;

use baza::{
    baza2::BazaManager,
    entities::{BLOBId, Document, DocumentType, InstanceId, Revision},
};
use rs_utils::{get_file_name, list_files, log::setup_logger, FsTransaction, SecretString};

#[derive(Parser)]
struct Cli {
    /// The directory of the arhiv
    arhiv_dir: String,

    /// Run without making any changes
    #[arg(short)]
    dry_run: bool,
}

fn main() -> Result<()> {
    setup_logger();

    let args = Cli::parse();

    let arhiv_dir = args.arhiv_dir;
    println!("Arhiv directory: {}", arhiv_dir);

    if args.dry_run {
        println!("Dry run");
    }

    let db_file_path = format!("{arhiv_dir}/arhiv.sqlite");
    let data_dir_path = format!("{arhiv_dir}/data");
    let downloads_dir_path = format!("{arhiv_dir}/downloads");

    let arhiv = Arhiv::new_desktop();

    // Ask for password and create new Arhiv instance
    println!("Enter new password for baza2:");
    let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH)?;
    if !args.dry_run {
        arhiv.baza.create(password)?;
    }

    // Create state using current instance id
    let instance_id = get_instance_id(&db_file_path)?;
    println!("Existing instance_id: {instance_id}");
    if !args.dry_run {
        arhiv.baza.create_state(instance_id)?;
    }

    // Insert document snapshots into storage
    let documents = get_all_snapshots(&db_file_path)?;
    println!("Selected {} document snapshots", documents.len());
    if !args.dry_run {
        arhiv
            .baza
            .dangerously_insert_snapshots_into_storage(&documents)?;
    }

    // Encrypt all data files
    let data_files = list_files(&data_dir_path)?;
    println!("Encrypting {} data files", data_files.len());
    if !args.dry_run {
        let mut fs_tx = FsTransaction::new();
        for file_path in data_files {
            let file_name = get_file_name(&file_path);
            let blob_id = BLOBId::from_file(&file_path)?;

            ensure!(
                file_name == blob_id.as_ref(),
                "File name must be a valid blob id"
            );

            arhiv
                .baza
                .dangerously_insert_blob_into_storage(&file_path)?;

            fs_tx.remove_file(&file_path)?;
        }
        fs_tx.commit()?;
    }

    // Cleanup

    if !args.dry_run {
        remove_file(&db_file_path)?;
    }
    println!("Removed db file {db_file_path}");

    if !args.dry_run {
        remove_dir(&downloads_dir_path)?;
    }
    println!("Removed downloads dir {downloads_dir_path}");

    Ok(())
}

fn extract_document(row: &Row) -> Result<Document> {
    let document_type: String = row.get("document_type")?;

    Ok(Document {
        id: {
            let id: String = row.get("id")?;

            id.into()
        },
        rev: {
            let rev: Value = row.get("rev")?;

            Revision::from_value(rev).context("failed to parse document rev")?
        },
        document_type: DocumentType::new(document_type),
        updated_at: row.get("updated_at")?,
        data: {
            let data: Value = row.get("data")?;

            data.try_into()?
        },
    })
}

fn get_all_snapshots(db_file_path: &str) -> Result<Vec<Document>> {
    let conn =
        rusqlite::Connection::open(db_file_path).context("Failed to open SQLite database")?;

    let mut stmt = conn
        .prepare("SELECT * FROM documents_snapshots GROUP BY id")
        .context("Failed to prepare statement")?;

    let documents: Vec<Document> = stmt
        .query_and_then([], extract_document)
        .context("Failed to query documents_snapshots")?
        .collect::<Result<_>>()?;

    Ok(documents)
}

fn get_instance_id(db_file_path: &str) -> Result<InstanceId> {
    let conn =
        rusqlite::Connection::open(db_file_path).context("Failed to open SQLite database")?;

    let mut stmt = conn
        .prepare(
            r#"SELECT json_extract(value, '$') FROM kvs WHERE key = '["settings","instance_id"]'"#,
        )
        .context("Failed to prepare statement")?;

    let instance_id: String = stmt
        .query_row([], |row| row.get(0))
        .context("Failed to query instance_id")?;

    Ok(InstanceId::from_string(instance_id))
}

fn prompt_password(min_length: usize) -> Result<SecretString> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Password (min {min_length} symbols):"))
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .validate_with(|input: &String| -> Result<(), String> {
            if input.chars().count() >= min_length {
                Ok(())
            } else {
                Err(format!("Password must be longer than {min_length}"))
            }
        })
        .interact()
        .map(|value| value.into())
        .context("Failed to prompt password")
}
