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

use anyhow::Result;
use tokio::sync::broadcast::Sender;

pub use connection::BazaConnection;
pub use dto::{BLOBSCount, DocumentsCount, ListPage};
pub use filter::{Conditions, Filter, OrderBy};
pub use kvs::{KvsConstKey, KvsEntry, KvsKey};
pub use migrations::{apply_db_migrations, create_db};

pub(crate) use sqlite_connection::{open_connection, vacuum};

use crate::{path_manager::PathManager, schema::DataSchema, BazaEvent};

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
