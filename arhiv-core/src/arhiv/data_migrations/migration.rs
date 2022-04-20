use std::borrow::Cow;

use anyhow::Result;

use crate::entities::Document;

pub trait DataMigration: Send + Sync {
    fn get_version(&self) -> u8;

    #[allow(clippy::ptr_arg)]
    fn update(&self, document: &mut Cow<Document>, data_dir: &str) -> Result<()>;
}
