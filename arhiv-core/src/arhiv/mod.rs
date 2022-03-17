mod backup;
pub(crate) mod db;
mod migrations;
mod status;
mod sync;

use anyhow::{anyhow, ensure, Context, Result};

use rs_utils::log;

use crate::{config::Config, definitions::get_standard_schema, entities::*, schema::DataSchema};

use self::db::{
    ArhivConnection, BLOBQueries, Filter, ListPage, MutableQueries, Queries, DB, SETTING_ARHIV_ID,
    SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME, SETTING_SCHEMA_VERSION,
};
use self::migrations::{apply_db_migrations, create_db, get_db_version};
use self::status::Status;

pub struct Arhiv {
    config: Config,
    pub(crate) db: DB,
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

        let db = DB::open(config.arhiv_root.to_string(), schema)?;

        let tx = db.get_tx()?;

        // ensure document schema is up to date
        tx.apply_migrations()?;

        {
            let schema_version = tx.get_setting(SETTING_SCHEMA_VERSION)?;

            ensure!(
                schema_version == db.schema.get_version(),
                "schema version {} is different from latest schema version {}",
                schema_version,
                db.schema.get_version(),
            );
        }

        // ensure computed data is up to date
        tx.compute_data().context("failed to compute data")?;

        tx.commit()?;

        log::debug!("Open arhiv in {}", config.arhiv_root);

        Ok(Arhiv { config, db })
    }

    pub fn create(
        config: Config,
        schema: DataSchema,
        arhiv_id: String,
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

        let schema_version = schema.get_version();

        let db = DB::open(config.arhiv_root.to_string(), schema)?;

        // TODO remove created arhiv if settings tx fails
        let tx = db.get_tx()?;

        // initial settings
        tx.set_setting(SETTING_ARHIV_ID, arhiv_id)?;
        tx.set_setting(SETTING_IS_PRIME, prime)?;
        tx.set_setting(SETTING_SCHEMA_VERSION, schema_version)?;
        tx.set_setting(SETTING_LAST_SYNC_TIME, chrono::MIN_DATETIME)?;

        tx.commit()?;

        Ok(Arhiv { config, db })
    }

    #[must_use]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn get_schema(&self) -> &DataSchema {
        &self.db.schema
    }

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.config.arhiv_root.to_string();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let conn = self.db.get_connection()?;

        let db_status = conn.get_db_status()?;
        let db_version = get_db_version(conn.get_connection())?;
        let schema_version = conn.get_setting(SETTING_SCHEMA_VERSION)?;
        let documents_count = conn.count_documents()?;
        let blobs_count = conn.count_blobs()?;
        let conflicts_count = conn.count_conflicts()?;
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            db_status,
            db_version,
            schema_version,
            documents_count,
            blobs_count,
            conflicts_count,
            last_update_time,
            debug_mode,
            root_dir,
        })
    }

    pub fn is_prime(&self) -> Result<bool> {
        let conn = self.db.get_connection()?;

        conn.get_setting(SETTING_IS_PRIME)
    }

    pub fn list_documents(&self, filter: impl AsRef<Filter>) -> Result<ListPage<Document>> {
        let conn = self.db.get_connection()?;

        conn.list_documents(filter.as_ref())
    }

    pub fn get_document(&self, id: impl Into<Id>) -> Result<Option<Document>> {
        let conn = self.db.get_connection()?;

        conn.get_document(&id.into())
    }

    pub fn must_get_document(&self, id: impl Into<Id>) -> Result<Document> {
        let id = id.into();

        self.get_document(&id)?
            .ok_or_else(|| anyhow!("Can't find document with id '{}'", id))
    }

    pub fn get_tx(&self) -> Result<ArhivConnection> {
        self.db.get_tx()
    }

    pub fn get_blob(&self, id: &BLOBId) -> Result<BLOB> {
        Ok(self.db.get_connection()?.get_blob(id))
    }
}
