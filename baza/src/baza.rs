use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};

use rs_utils::log;

pub use crate::events::BazaEvent;
use crate::{
    db::BazaConnection,
    path_manager::PathManager,
    schema::{DataMigrations, DataSchema},
    DocumentExpert, DB,
};

pub struct BazaOptions {
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
    fn new(root_dir: String, schema: DataSchema) -> Self {
        let path_manager = PathManager::new(root_dir);

        Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            events: channel(42),
        }
    }

    pub fn create(options: BazaOptions) -> Result<Baza> {
        let baza = Baza::new(options.root_dir, options.schema);

        log::info!(
            "Initializing {} Baza in {}",
            baza.get_app_name(),
            baza.path_manager.root_dir
        );
        baza.get_db().init(&options.migrations)?;

        Ok(baza)
    }

    pub fn open(options: BazaOptions) -> Result<Baza> {
        let baza = Baza::new(options.root_dir, options.schema);

        baza.path_manager.assert_dirs_exist()?;
        baza.path_manager.assert_db_file_exists()?;

        let db = baza.get_db();

        db.apply_db_migrations()?;
        db.apply_data_migrations(&options.migrations)?;

        log::debug!(
            "Opened {} Baza in {}",
            baza.get_app_name(),
            baza.path_manager.root_dir
        );

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
    pub fn get_document_expert(&self) -> DocumentExpert {
        DocumentExpert::new(self.get_schema())
    }

    #[must_use]
    pub fn get_app_name(&self) -> &str {
        self.schema.get_app_name()
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
