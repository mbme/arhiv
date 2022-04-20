mod migration;
mod v1;
mod v2;

use std::borrow::Cow;
use std::time::Instant;

use anyhow::{ensure, Context, Result};
use lazy_static::lazy_static;
use rs_utils::log;

use crate::ArhivConnection;

use super::db::SETTING_DATA_VERSION;

use self::migration::DataMigration;
use self::v1::DataSchema1;
use self::v2::DataSchema2;

lazy_static! {
    static ref MIGRATIONS: Vec<Box<dyn DataMigration>> = vec![ //
        Box::new(DataSchema1),
        Box::new(DataSchema2),
    ];

    static ref DATA_VERSION: u8 = MIGRATIONS
        .iter()
        .fold(0, |latest_version, migration| {
            migration.get_version().max(latest_version)
        });
}

#[must_use]
pub fn get_latest_data_version() -> u8 {
    *DATA_VERSION
}

pub(crate) fn apply_data_migrations(conn: &ArhivConnection) -> Result<()> {
    let data_version = conn.get_setting(&SETTING_DATA_VERSION)?;
    let latest_data_version = get_latest_data_version();

    ensure!(
        data_version <= latest_data_version,
        "data_version {} is bigger than latest data version {}",
        data_version,
        latest_data_version
    );

    let migrations = MIGRATIONS
        .iter()
        .filter(|migration| migration.get_version() > data_version)
        .collect::<Vec<_>>();

    if migrations.is_empty() {
        log::debug!("no schema migrations to apply");

        return Ok(());
    }

    log::info!("{} schema migrations to apply", migrations.len());

    let mut stmt = conn
        .get_connection()
        .prepare("SELECT rowid FROM documents_snapshots")?;

    let row_ids = stmt
        .query_and_then([], |row| row.get(0).context("failed to get arg 0"))
        .context("failed to query documents snapshots")?
        .collect::<Result<Vec<i64>>>()?;

    let now = Instant::now();
    let mut rows_count = 0;
    for rowid in row_ids {
        let document = conn.get_document_by_rowid(rowid)?;
        let mut document = Cow::Borrowed(&document);

        let data_dir = &conn.get_path_manager().data_dir;
        for migration in &migrations {
            migration.update(&mut document, data_dir)?;
        }

        // update document only if it has been mutated
        if let Cow::Owned(document) = document {
            conn.put_or_replace_document(&document, true)?;
            rows_count += 1;
        }
    }

    conn.set_setting(&SETTING_DATA_VERSION, &latest_data_version)?;

    log::info!(
        "Migrated {} rows in {} seconds",
        rows_count,
        now.elapsed().as_secs_f32()
    );

    log::info!("Finished data migration");

    Ok(())
}
