use anyhow::Result;

use crate::entities::Document;

pub trait DataMigration: Send + Sync {
    fn get_version(&self) -> u8;

    fn update(&self, document: &mut Document) -> Result<()>;
}
