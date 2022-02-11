mod backup;
pub(crate) mod db;
mod migrations;
mod status;
mod sync;

use anyhow::{anyhow, ensure, Context, Result};
use chrono::Utc;

use rs_utils::log;

use crate::{
    config::Config, definitions::get_standard_schema, entities::*, schema::DataSchema, Validator,
};

use self::db::{
    ArhivTransaction, BLOBQueries, Filter, ListPage, MutableBLOBQueries, MutableQueries, Queries,
    DB, SETTING_ARHIV_ID, SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME, SETTING_SCHEMA_VERSION,
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

        // ensure document schema is up to date
        db.apply_migrations()?;

        {
            let schema_version = db.get_connection()?.get_setting(SETTING_SCHEMA_VERSION)?;

            ensure!(
                schema_version == db.schema.get_version(),
                "schema version {} is different from latest schema version {}",
                schema_version,
                db.schema.get_version(),
            );
        }

        // ensure computed data is up to date
        db.compute_data().context("failed to compute data")?;

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

    pub fn get_tx(&self) -> Result<ArhivTransaction> {
        self.db.get_tx()
    }

    pub fn stage_document(&self, document: &mut Document) -> Result<()> {
        let mut tx = self.get_tx()?;

        self.tx_stage_document(document, &mut tx)?;

        tx.commit()?;

        Ok(())
    }

    pub fn tx_stage_document(
        &self,
        document: &mut Document,
        tx: &mut ArhivTransaction,
    ) -> Result<()> {
        log::debug!("Staging document {}", &document.id);

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        let prev_document = tx.get_document(&document.id)?;

        let data_description = self
            .get_schema()
            .get_data_description(&document.document_type)?;

        Validator::default().validate(
            &document.data,
            prev_document.as_ref().map(|document| &document.data),
            data_description,
            tx,
        )?;

        if let Some(prev_document) = prev_document {
            log::debug!("Updating existing document {}", &document.id);

            document.rev = Revision::STAGING;

            if prev_document.rev == Revision::STAGING {
                ensure!(
                    document.prev_rev == prev_document.prev_rev,
                    "document prev_rev {} is different from the staged document prev_rev {}",
                    document.prev_rev,
                    prev_document.prev_rev
                );
            } else {
                // we're going to modify committed document
                // so we need to save its revision as prev_rev of the new document
                document.prev_rev = prev_document.rev;
            }

            ensure!(
                document.document_type == prev_document.document_type,
                "document type '{}' is different from the type '{}' of existing document",
                document.document_type,
                prev_document.document_type
            );

            ensure!(
                document.created_at == prev_document.created_at,
                "document created_at '{}' is different from the created_at '{}' of existing document",
                document.created_at,
                prev_document.created_at
            );

            ensure!(
                document.updated_at == prev_document.updated_at,
                "document updated_at '{}' is different from the updated_at '{}' of existing document",
                document.updated_at,
                prev_document.updated_at
            );

            document.updated_at = Utc::now();
        } else {
            log::debug!("Creating new document {}", &document.id);

            document.rev = Revision::STAGING;
            document.prev_rev = Revision::STAGING;

            let now = Utc::now();
            document.created_at = now;
            document.updated_at = now;
        }

        tx.put_document(document)?;

        log::info!("saved document {}", document);

        Ok(())
    }

    pub fn erase_document(&self, id: &Id) -> Result<()> {
        let mut document = self
            .get_document(id)?
            .ok_or_else(|| anyhow!("can't find document {}", &id))?;

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        let tx = self.db.get_tx()?;

        document.document_type = ERASED_DOCUMENT_TYPE.to_string();
        document.rev = Revision::STAGING;
        document.prev_rev = Revision::STAGING;
        document.data = DocumentData::new();
        document.updated_at = Utc::now();

        tx.put_document(&document)?;

        tx.commit()?;

        log::info!("erased document {}", document);

        Ok(())
    }

    pub fn add_blob(&self, file_path: &str, move_file: bool) -> Result<BLOBId> {
        let mut tx = self.get_tx()?;

        let id = tx.add_blob(file_path, move_file)?;

        tx.commit()?;

        Ok(id)
    }

    #[allow(clippy::unused_self)]
    pub fn tx_add_blob(
        &self,
        file_path: &str,
        move_file: bool,
        tx: &mut ArhivTransaction,
    ) -> Result<BLOBId> {
        tx.add_blob(file_path, move_file)
    }

    pub fn get_blob(&self, id: &BLOBId) -> Result<BLOB> {
        Ok(self.db.get_connection()?.get_blob(id))
    }
}
