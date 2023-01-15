use std::sync::Arc;

use anyhow::{Context, Result};

use rs_utils::log;

use crate::{
    db::{vacuum, BazaConnection, Filter, ListPage, SETTING_DATA_VERSION},
    db_migrations::{apply_db_migrations, create_db},
    entities::*,
    path_manager::PathManager,
    schema::{get_latest_data_version, DataMigrations, DataSchema},
};

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    pub(crate) data_version: u8,
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
        };

        // TODO remove created arhiv if settings tx fails
        let tx = baza.get_tx()?;

        tx.set_setting(&SETTING_DATA_VERSION, &baza.data_version)?;

        tx.commit()?;

        Ok(baza)
    }

    pub fn cleanup(&self) -> Result<()> {
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

    pub fn get_connection(&self) -> Result<BazaConnection> {
        BazaConnection::new(self.path_manager.clone(), self.schema.clone())
    }

    pub fn get_tx(&self) -> Result<BazaConnection> {
        BazaConnection::new_tx(self.path_manager.clone(), self.schema.clone())
    }

    #[must_use]
    pub fn get_path_manager(&self) -> &PathManager {
        &self.path_manager
    }

    #[must_use]
    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    pub fn list_documents(&self, filter: impl AsRef<Filter>) -> Result<ListPage> {
        let conn = self.get_connection()?;

        conn.list_documents(filter.as_ref())
    }

    pub fn get_document(&self, id: impl Into<Id>) -> Result<Option<Document>> {
        let conn = self.get_connection()?;

        conn.get_document(&id.into())
    }

    pub fn get_blob(&self, id: &BLOBId) -> Result<BLOB> {
        let conn = self.get_connection()?;

        Ok(conn.get_blob(id))
    }
}
