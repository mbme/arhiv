use anyhow::Result;

use baza::{
    schema::{DataMigrations, DataSchema},
    Baza, BazaConnection, SETTING_DATA_VERSION,
};
use rs_utils::{get_crate_version, MIN_TIMESTAMP};

use crate::{
    config::Config,
    data_migrations::get_data_migrations,
    definitions::get_standard_schema,
    settings::SETTING_LAST_SYNC_TIME,
    status::{DbStatus, Status},
};

pub struct Arhiv {
    pub baza: Baza,
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

        Ok(Arhiv { baza, config })
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

        let tx = baza.get_tx()?;

        tx.kvs_const_set(SETTING_LAST_SYNC_TIME, &MIN_TIMESTAMP)?;

        tx.commit()?;

        Ok(Arhiv { baza, config })
    }

    pub fn get_config(&self) -> &Config {
        &self.config
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
