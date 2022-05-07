use std::sync::Arc;

use anyhow::{anyhow, Context, Result};

use rs_utils::log;

use crate::path_manager::PathManager;
use crate::{config::Config, definitions::get_standard_schema, entities::*, schema::DataSchema};

use self::data_migrations::{apply_data_migrations, get_latest_data_version};
use self::db::{
    vacuum, ArhivConnection, Filter, ListPage, SETTING_ARHIV_ID, SETTING_DATA_VERSION,
    SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME,
};
use self::db_migrations::{apply_db_migrations, create_db};
use self::status::Status;

mod backup;
mod data_migrations;
pub(crate) mod db;
mod db_migrations;
mod status;
mod sync;

pub struct Arhiv {
    config: Config,
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    data_version: u8,
}

impl Arhiv {
    #[must_use]
    pub fn must_open() -> Arhiv {
        Arhiv::open().expect("must be able to open arhiv")
    }

    pub fn open() -> Result<Arhiv> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();

        Arhiv::open_with_options(config, schema)
    }

    pub fn open_with_options(config: Config, schema: DataSchema) -> Result<Arhiv> {
        // ensure DB schema is up to date
        apply_db_migrations(&config.arhiv_root)
            .context("failed to apply migrations to arhiv db")?;

        let path_manager = PathManager::new(config.arhiv_root.to_string());
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        let arhiv = Arhiv {
            config,
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            data_version: get_latest_data_version(),
        };

        let tx = arhiv.get_tx()?;

        // ensure data is up to date
        apply_data_migrations(&tx).context("failed to apply data migrations to arhiv db")?;

        // ensure computed data is up to date
        tx.compute_data().context("failed to compute data")?;

        tx.commit()?;

        log::debug!("Open arhiv in {}", arhiv.config.arhiv_root);

        Ok(arhiv)
    }

    pub fn create(
        config: Config,
        schema: DataSchema,
        arhiv_id: &str,
        prime: bool,
    ) -> Result<Arhiv> {
        log::info!(
            "Initializing {} arhiv '{}' in {}",
            if prime { "prime" } else { "replica" },
            arhiv_id,
            config.arhiv_root
        );

        create_db(&config.arhiv_root)?;
        log::info!("Created arhiv in {}", config.arhiv_root);

        let path_manager = PathManager::new(config.arhiv_root.to_string());

        let arhiv = Arhiv {
            config,
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            data_version: get_latest_data_version(),
        };

        // TODO remove created arhiv if settings tx fails
        let tx = arhiv.get_tx()?;

        // initial settings
        tx.set_setting(&SETTING_ARHIV_ID, &arhiv_id.to_string())?;
        tx.set_setting(&SETTING_IS_PRIME, &prime)?;
        tx.set_setting(&SETTING_DATA_VERSION, &arhiv.data_version)?;
        tx.set_setting(&SETTING_LAST_SYNC_TIME, &chrono::MIN_DATETIME)?;

        tx.commit()?;

        Ok(arhiv)
    }

    fn cleanup(&self) -> Result<()> {
        log::debug!("Initiating cleanup...");

        vacuum(&self.path_manager.db_file)?;

        {
            let mut tx = self.get_tx()?;
            tx.remove_orphaned_blobs()?;
            tx.commit()?;
        }

        log::debug!("Cleanup completed");

        Ok(())
    }

    fn get_connection(&self) -> Result<ArhivConnection> {
        ArhivConnection::new(self.path_manager.clone(), self.schema.clone())
    }

    pub fn get_tx(&self) -> Result<ArhivConnection> {
        ArhivConnection::new_tx(self.path_manager.clone(), self.schema.clone())
    }

    #[must_use]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    pub fn get_status(&self) -> Result<Status> {
        let conn = self.get_connection()?;

        conn.get_status()
    }

    pub fn is_prime(&self) -> Result<bool> {
        let conn = self.get_connection()?;

        conn.get_setting(&SETTING_IS_PRIME)
    }

    pub fn list_documents(&self, filter: impl AsRef<Filter>) -> Result<ListPage> {
        let conn = self.get_connection()?;

        conn.list_documents(filter.as_ref())
    }

    pub fn get_document(&self, id: impl Into<Id>) -> Result<Option<Document>> {
        let conn = self.get_connection()?;

        conn.get_document(&id.into())
    }

    pub fn must_get_document(&self, id: impl Into<Id>) -> Result<Document> {
        let id = id.into();

        self.get_document(&id)?
            .ok_or_else(|| anyhow!("Can't find document with id '{}'", id))
    }

    pub fn get_blob(&self, id: &BLOBId) -> Result<BLOB> {
        let conn = self.get_connection()?;

        Ok(conn.get_blob(id))
    }
}
