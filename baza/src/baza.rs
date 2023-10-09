use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};

use rs_utils::{log, mdns::MDNSService, MIN_TIMESTAMP};

pub use crate::events::BazaEvent;
use crate::{
    db::{vacuum, BazaConnection, SETTING_DATA_VERSION},
    db_migrations::{apply_db_migrations, create_db},
    path_manager::PathManager,
    schema::{get_latest_data_version, DataMigrations, DataSchema},
    sync::InstanceId,
    DEBUG_MODE, SETTING_INSTANCE_ID, SETTING_LAST_SYNC_TIME,
};

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    data_version: u8,
    mdns_service: OnceLock<MDNSService>,
    events: (Sender<BazaEvent>, Receiver<BazaEvent>),
}

impl Baza {
    pub fn open(root_dir: String, schema: DataSchema, migrations: DataMigrations) -> Result<Baza> {
        // ensure DB schema is up to date
        apply_db_migrations(&root_dir).context("failed to apply migrations to Baza db")?;

        let path_manager = PathManager::new(root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        let baza = Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            data_version: get_latest_data_version(&migrations),
            events: channel(42),
            mdns_service: Default::default(),
        };

        let tx = baza.get_tx()?;

        // ensure data is up to date
        tx.apply_data_migrations(&migrations)
            .context("failed to apply data migrations to Baza db")?;

        // ensure computed data is up to date
        tx.compute_data().context("failed to compute data")?;

        tx.commit()?;

        log::debug!("Open Baza in {}", &baza.path_manager.root_dir);

        Ok(baza)
    }

    pub fn create(
        root_dir: String,
        schema: DataSchema,
        migrations: DataMigrations,
    ) -> Result<Baza> {
        log::info!("Initializing Baza in {root_dir}");

        create_db(&root_dir)?;
        log::info!("Created Baza in {}", root_dir);

        let path_manager = PathManager::new(root_dir);

        let baza = Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            data_version: get_latest_data_version(&migrations),
            events: channel(42),
            mdns_service: Default::default(),
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

    fn init_mdns_service(&self) -> Result<MDNSService> {
        let instance_id = self
            .get_connection()
            .and_then(|conn| conn.get_instance_id())
            .context("failed to read instance_id")?;

        let app_name = self.get_name();

        let mut service_name = format!("_{app_name}-baza");
        if DEBUG_MODE {
            service_name.push_str("-debug");
        }

        MDNSService::new(service_name, instance_id)
    }

    pub fn get_mdns_service(&self) -> &MDNSService {
        self.mdns_service
            .get_or_init(|| self.init_mdns_service().expect("must init MDNS service"))
    }

    pub fn stop(mut self) {
        if let Some(ref mut mdns_service) = self.mdns_service.get_mut() {
            mdns_service.shutdown();
        }
    }

    #[must_use]
    pub fn get_events_channel(&self) -> Receiver<BazaEvent> {
        self.events.0.subscribe()
    }

    pub fn publish_event(&self, event: BazaEvent) -> Result<()> {
        self.events
            .0
            .send(event)
            .context("failed to publish baza event")?;

        Ok(())
    }
}
