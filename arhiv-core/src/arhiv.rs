use std::sync::Arc;

use anyhow::Result;

use baza::{
    schema::{DataMigrations, DataSchema},
    sync::SyncService,
    Baza, BazaConnection, SETTING_DATA_VERSION, SETTING_LAST_SYNC_TIME,
};
use rs_utils::get_crate_version;

use crate::{
    config::Config,
    data_migrations::get_data_migrations,
    definitions::get_standard_schema,
    status::{DbStatus, Status},
};

pub struct Arhiv {
    pub baza: Arc<Baza>,
    pub(crate) config: Config,
}

impl Arhiv {
    #[must_use]
    pub fn must_open() -> Arhiv {
        Arhiv::open().expect("must be able to open arhiv")
    }

    pub fn open() -> Result<Arhiv> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        let baza = Baza::open(config.arhiv_root.clone(), schema, data_migrations)?;

        Ok(Arhiv {
            baza: Arc::new(baza),
            config,
        })
    }

    pub fn create() -> Result<Self> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        Arhiv::create_with_options(config, schema, data_migrations)
    }

    pub fn create_with_options(
        config: Config,
        schema: DataSchema,
        data_migrations: DataMigrations,
    ) -> Result<Self> {
        let baza = Baza::create(config.arhiv_root.clone(), schema, data_migrations)?;

        Ok(Arhiv {
            baza: Arc::new(baza),
            config,
        })
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_sync_service(&self) -> Result<SyncService> {
        let mut sync_service = SyncService::new(&self.baza);

        // TODO start MDNS client

        sync_service.parse_network_agents(&self.config.static_peers)?;

        Ok(sync_service)
    }
}

pub trait BazaConnectionExt {
    fn get_db_status(&self) -> Result<DbStatus>;

    fn get_status(&self) -> Result<Status>;
}

impl BazaConnectionExt for BazaConnection {
    fn get_db_status(&self) -> Result<DbStatus> {
        Ok(DbStatus {
            data_version: self.kvs_const_must_get(SETTING_DATA_VERSION)?,
            db_rev: self.get_db_rev()?,
            last_sync_time: self.kvs_const_must_get(SETTING_LAST_SYNC_TIME)?,
        })
    }

    fn get_status(&self) -> Result<Status> {
        let root_dir = self.get_path_manager().root_dir.clone();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let db_status = self.get_db_status()?;
        let db_version = self.get_db_version()?;
        let data_version = self.kvs_const_must_get(SETTING_DATA_VERSION)?;
        let documents_count = self.count_documents()?;
        let blobs_count = self.count_blobs()?;
        let conflicts_count = self.get_coflicting_documents()?.len();
        let last_update_time = self.get_last_update_time()?;

        Ok(Status {
            app_version: get_crate_version().to_string(),
            db_status,
            db_version,
            data_version,
            documents_count,
            blobs_count,
            conflicts_count,
            last_update_time,
            debug_mode,
            root_dir,
        })
    }
}
