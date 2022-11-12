use std::borrow::Cow;

use anyhow::Result;

use crate::entities::Document;

pub trait DataMigration {
    fn get_version(&self) -> u8;

    #[allow(clippy::ptr_arg)]
    fn update(&self, document: &mut Cow<Document>, data_dir: &str) -> Result<()>;
}

pub type DataMigrations = Vec<Box<dyn DataMigration>>;

#[must_use]
pub fn get_latest_data_version(migrations: &DataMigrations) -> u8 {
    migrations.iter().fold(0, |latest_version, migration| {
        migration.get_version().max(latest_version)
    })
}
