use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};

use rs_utils::log;

pub use crate::events::BazaEvent;
use crate::{
    db::BazaConnection,
    path_manager::PathManager,
    schema::{DataMigrations, DataSchema},
    DB,
};

pub struct BazaOptions {
    pub create: bool,
    pub migrations: DataMigrations,
    pub root_dir: String,
    pub schema: DataSchema,
}

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    events: (Sender<BazaEvent>, Receiver<BazaEvent>),
}

impl Baza {
    pub fn open(options: BazaOptions) -> Result<Baza> {
        let path_manager = PathManager::new(options.root_dir.clone());

        let baza = Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(options.schema),
            events: channel(42),
        };

        if options.create {
            log::info!("Initializing Baza in {}", options.root_dir);
            baza.get_db().init(&options.migrations)?;
        } else {
            baza.path_manager.assert_dirs_exist()?;
            baza.path_manager.assert_db_file_exists()?;

            let db = baza.get_db();

            db.apply_db_migrations()?;
            db.apply_data_migrations(&options.migrations)?;
        }

        log::debug!("Open Baza in {}", &baza.path_manager.root_dir);

        Ok(baza)
    }

    #[must_use]
    pub fn get_db(&self) -> DB {
        DB::new(
            self.path_manager.clone(),
            self.schema.clone(),
            self.events.0.clone(),
        )
    }

    pub fn get_connection(&self) -> Result<BazaConnection> {
        self.get_db().get_connection()
    }

    pub fn get_tx(&self) -> Result<BazaConnection> {
        self.get_db().get_tx()
    }

    #[must_use]
    pub fn get_path_manager(&self) -> &PathManager {
        &self.path_manager
    }

    #[must_use]
    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    #[must_use]
    pub fn get_name(&self) -> &str {
        self.schema.get_name()
    }

    #[must_use]
    pub fn get_events_channel(&self) -> Receiver<BazaEvent> {
        self.events.0.subscribe()
    }

    #[must_use]
    pub(crate) fn get_events_sender(&self) -> Sender<BazaEvent> {
        self.events.0.clone()
    }

    pub(crate) fn publish_event(&self, event: BazaEvent) -> Result<()> {
        self.events
            .0
            .send(event)
            .context("failed to publish baza event")?;

        Ok(())
    }
}
