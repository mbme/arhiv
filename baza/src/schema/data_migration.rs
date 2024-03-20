use std::borrow::Cow;

use anyhow::Result;

use crate::{entities::Document, BazaConnection};

pub trait DataMigration: Send + Sync {
    fn get_version(&self) -> u8;

    #[allow(clippy::ptr_arg)]
    fn update(&self, document: &mut Cow<Document>, conn: &BazaConnection) -> Result<()>;
}

pub type DataMigrations = Vec<Box<dyn DataMigration>>;
