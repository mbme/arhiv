mod migration;
mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;

use anyhow::{ensure, Context, Result};
use rusqlite::{Connection, OpenFlags};

use rs_utils::{log, FsTransaction, TempFile};

use crate::db::open_connection;
use crate::path_manager::PathManager;

use self::migration::DBMigration;
use self::v1::MigrationV1;
use self::v2::MigrationV2;
use self::v3::MigrationV3;
use self::v4::MigrationV4;
use self::v5::MigrationV5;
use self::v6::MigrationV6;

pub fn get_db_version(conn: &Connection) -> Result<u8> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .context("failed to read PRAGMA user_version")
}

pub fn create_db(root_dir: impl Into<String>) -> Result<()> {
    let latest_migration = get_db_migrations()
        .into_iter()
        .reduce(|latest_migration, migration| {
            if migration.get_version() > latest_migration.get_version() {
                migration
            } else {
                latest_migration
            }
        })
        .expect("must have at least 1 migration");

    create_db_with_schema(root_dir, &*latest_migration)
}

fn create_db_with_schema(root_dir: impl Into<String>, migration: &dyn DBMigration) -> Result<()> {
    let new_db_pm = PathManager::new(root_dir.into());
    new_db_pm.create_dirs()?;

    let new_conn = Connection::open_with_flags(
        &new_db_pm.db_file,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    new_conn.execute_batch(migration.get_schema())?;

    new_conn.pragma_update(None, "user_version", migration.get_version())?;

    Ok(())
}

fn get_db_migrations() -> Vec<Box<dyn DBMigration>> {
    vec![
        //
        Box::new(MigrationV1),
        Box::new(MigrationV2),
        Box::new(MigrationV3),
        Box::new(MigrationV4),
        Box::new(MigrationV5),
        Box::new(MigrationV6),
    ]
}

pub fn apply_db_migrations(root_dir: impl Into<String>) -> Result<bool> {
    let root_dir = root_dir.into();

    let db_pm = PathManager::new(root_dir);

    let mut db_version = {
        let conn = open_connection(&db_pm.db_file, false)?;

        get_db_version(&conn)?
    };

    let migrations = get_db_migrations();
    let max_db_version = migrations.iter().fold(0, |max_db_version, migration| {
        migration.get_version().max(max_db_version)
    });
    ensure!(
        db_version <= max_db_version,
        "DB version {} is greater than max DB version {}",
        db_version,
        max_db_version
    );

    // check if upgrade is needed
    if db_version == max_db_version {
        log::debug!(
            "DB version {} matches max DB version, nothing to upgrade",
            db_version
        );

        return Ok(false);
    }

    log::warn!(
        "DB version {}, starting upgrade to version {}",
        db_version,
        max_db_version,
    );

    // while not max version:
    //   create temp dir
    //   run upgrade(old db, new file)
    //   replace old db with new file
    //   remove temp dir
    for migration in migrations {
        let upgrade_version = migration.get_version();

        // skip irrelevant upgrades
        if db_version >= upgrade_version {
            continue;
        }

        log::info!("Upgrading db to version {}...", upgrade_version);

        let temp_dir = TempFile::new_with_details(format!("DB-upgrade-{upgrade_version}-"), "");

        let mut fs_tx = FsTransaction::new();

        let new_db_pm = PathManager::new(temp_dir.as_ref().to_string());
        create_db_with_schema(temp_dir.as_ref().to_string(), &*migration)?;

        {
            let new_conn = open_connection(&new_db_pm.db_file, true)?;

            new_conn.execute_batch(&format!("ATTACH DATABASE '{}' AS 'old_db'", db_pm.db_file,))?;

            new_conn.execute_batch("BEGIN DEFERRED")?;

            migration
                .apply(&new_conn, &mut fs_tx, &db_pm.data_dir)
                .context("failed to apply migration")?;
            migration
                .test(&new_conn, &db_pm.data_dir)
                .context("migration test failed")?;

            new_conn.execute_batch("COMMIT")?;

            new_conn.execute("VACUUM", [])?;
        }

        fs_tx.move_file(&new_db_pm.db_file, &db_pm.db_file, false)?;
        fs_tx.commit()?;

        log::warn!(
            "Upgraded db from version {} to version {}",
            db_version,
            upgrade_version
        );

        db_version = upgrade_version;
    }

    log::warn!("Done");

    Ok(true)
}
