use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};

use rs_utils::{log, MIN_TIMESTAMP};

pub use crate::events::BazaEvent;
use crate::{
    db::{vacuum, BazaConnection, SETTING_DATA_VERSION},
    db_migrations::{apply_db_migrations, create_db},
    path_manager::PathManager,
    schema::{get_latest_data_version, DataMigrations, DataSchema},
    sync::InstanceId,
    SETTING_INSTANCE_ID, SETTING_LAST_SYNC_TIME,
};

pub struct BazaOptions {
    pub migrations: DataMigrations,
    pub root_dir: String,
    pub schema: DataSchema,
}

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    data_version: u8,
    events: (Sender<BazaEvent>, Receiver<BazaEvent>),
}

impl Baza {
    pub fn open(options: BazaOptions) -> Result<Baza> {
        // ensure DB schema is up to date
        apply_db_migrations(&options.root_dir).context("failed to apply migrations to Baza db")?;

        let path_manager = PathManager::new(options.root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        let events = channel(42);
        let baza = Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(options.schema),
            data_version: get_latest_data_version(&options.migrations),
            events,
        };

        let tx = baza.get_tx()?;

        // ensure data is up to date
        tx.apply_data_migrations(&options.migrations)
            .context("failed to apply data migrations to Baza db")?;

        // ensure computed data is up to date
        tx.compute_data().context("failed to compute data")?;

        tx.commit()?;

        log::debug!("Open Baza in {}", &baza.path_manager.root_dir);

        Ok(baza)
    }

    pub fn create(options: BazaOptions) -> Result<Baza> {
        log::info!("Initializing Baza in {}", options.root_dir);

        create_db(&options.root_dir)?;
        log::info!("Created Baza in {}", options.root_dir);

        let path_manager = PathManager::new(options.root_dir);

        let events = channel(42);
        let baza = Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(options.schema),
            data_version: get_latest_data_version(&options.migrations),
            events,
        };

        // TODO remove created arhiv if settings tx fails
        let tx = baza.get_tx()?;

        tx.kvs_const_set(SETTING_DATA_VERSION, &baza.data_version)?;
        tx.kvs_const_set(SETTING_INSTANCE_ID, &InstanceId::new())?;
        tx.kvs_const_set(SETTING_LAST_SYNC_TIME, &MIN_TIMESTAMP)?;

        tx.commit()?;

        Ok(baza)
    }

    pub fn cleanup(&self) -> Result<()> {
        log::debug!("Initiating cleanup...");

        vacuum(&self.path_manager.db_file)?;

        log::debug!("Cleanup completed");

        Ok(())
    }

    pub fn get_connection(&self) -> Result<BazaConnection> {
        BazaConnection::new(self.path_manager.clone(), self.schema.clone())
    }

    pub fn get_tx(&self) -> Result<BazaConnection> {
        BazaConnection::new_tx(
            self.path_manager.clone(),
            self.schema.clone(),
            self.events.0.clone(),
        )
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
