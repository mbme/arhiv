use std::borrow::Cow;

use anyhow::Result;

use crate::{entities::Document, BazaConnection};

pub trait DataMigration {
    fn get_version(&self) -> u8;

    #[allow(clippy::ptr_arg)]
    fn update(&self, document: &mut Cow<Document>, conn: &BazaConnection) -> Result<()>;
}

pub type DataMigrations = Vec<Box<dyn DataMigration>>;

#[must_use]
pub fn get_latest_data_version(migrations: &DataMigrations) -> u8 {
    migrations.iter().fold(0, |latest_version, migration| {
        migration.get_version().max(latest_version)
    })
}

#[must_use]
pub fn get_min_data_migration_version(migrations: &DataMigrations) -> u8 {
    migrations
        .iter()
        .fold(u8::MAX, |latest_version, migration| {
            migration.get_version().min(latest_version)
        })
}
