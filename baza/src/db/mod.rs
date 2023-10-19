mod connection;
mod dto;
mod filter;
mod kvs;
pub mod locks;
mod migrations;
mod query_builder;
pub mod settings;
mod sqlite_connection;
mod utils;

use std::sync::Arc;

use anyhow::{Context, Result};
use rs_utils::MIN_TIMESTAMP;
use tokio::sync::broadcast::Sender;

pub use connection::BazaConnection;
pub use dto::{BLOBSCount, DocumentsCount, ListPage};
pub use filter::{Conditions, Filter, OrderBy};
pub use kvs::{KvsConstKey, KvsEntry, KvsKey};

use migrations::{apply_db_migrations, create_db};
use sqlite_connection::{open_connection, vacuum};

use crate::{
    entities::InstanceId,
    path_manager::PathManager,
    schema::{get_latest_data_version, DataMigrations, DataSchema},
    BazaEvent,
};

pub struct DB {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    event_sender: Sender<BazaEvent>,
}

impl DB {
    pub fn new(
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
        event_sender: Sender<BazaEvent>,
    ) -> Self {
        DB {
            path_manager,
            schema,
            event_sender,
        }
    }

    pub(crate) fn init(&self, migrations: &DataMigrations) -> Result<()> {
        create_db(&self.path_manager.root_dir)?;

        let tx = self.get_tx()?;

        tx.set_data_version(get_latest_data_version(migrations))?;
        tx.set_instance_id(&InstanceId::new())?;
        tx.set_last_sync_time(&MIN_TIMESTAMP)?;

        tx.commit()?;

        Ok(())
    }

    /// ensure DB schema is up to date
    pub(crate) fn apply_db_migrations(&self) -> Result<()> {
        apply_db_migrations(&self.path_manager.root_dir)
            .context("failed to apply migrations to Baza db")?;

        Ok(())
    }

    pub(crate) fn apply_data_migrations(&self, data_migrations: &DataMigrations) -> Result<()> {
        let tx = self.get_tx()?;

        // ensure data is up to date
        tx.apply_data_migrations(data_migrations)
            .context("failed to apply data migrations to Baza db")?;

        // ensure computed data is up to date
        tx.compute_data().context("failed to compute data")?;

        tx.commit()?;
        Ok(())
    }

    pub(crate) fn vacuum(&self) -> Result<()> {
        vacuum(&self.path_manager.db_file)?;

        Ok(())
    }

    pub fn get_connection(&self) -> Result<BazaConnection> {
        BazaConnection::new(self.path_manager.clone(), self.schema.clone())
    }

    pub fn get_tx(&self) -> Result<BazaConnection> {
        BazaConnection::new_tx(
            self.path_manager.clone(),
            self.schema.clone(),
            self.event_sender.clone(),
        )
    }
}
