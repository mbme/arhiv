use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, bail, ensure, Context, Result};
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    Connection, Error as RusqliteError, OpenFlags,
};
use serde_json::Value;

use rs_utils::log;

use crate::path_manager::PathManager;
use crate::{config::Config, definitions::get_standard_schema, entities::*, schema::DataSchema};

use self::db::{
    ArhivConnection, Filter, ListPage, SETTING_ARHIV_ID, SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME,
    SETTING_SCHEMA_VERSION,
};
use self::migrations::{apply_db_migrations, create_db};
use self::status::Status;

mod backup;
pub(crate) mod db;
mod migrations;
mod status;
mod sync;

pub struct Arhiv {
    config: Config,
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
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
        };

        let tx = arhiv.get_tx()?;

        // ensure document schema is up to date
        tx.apply_migrations()?;

        {
            let schema_version = tx.get_setting(&SETTING_SCHEMA_VERSION)?;

            ensure!(
                schema_version == arhiv.schema.get_version(),
                "schema version {} is different from latest schema version {}",
                schema_version,
                arhiv.schema.get_version(),
            );
        }

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

        let schema_version = schema.get_version();

        let path_manager = PathManager::new(config.arhiv_root.to_string());

        let arhiv = Arhiv {
            config,
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
        };

        // TODO remove created arhiv if settings tx fails
        let tx = arhiv.get_tx()?;

        // initial settings
        tx.set_setting(&SETTING_ARHIV_ID, &arhiv_id.to_string())?;
        tx.set_setting(&SETTING_IS_PRIME, &prime)?;
        tx.set_setting(&SETTING_SCHEMA_VERSION, &schema_version)?;
        tx.set_setting(&SETTING_LAST_SYNC_TIME, &chrono::MIN_DATETIME)?;

        tx.commit()?;

        Ok(arhiv)
    }

    fn open_connection(&self, mutable: bool) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path_manager.db_file,
            if mutable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            },
        )
        .context("failed to open connection")?;

        conn.pragma_update(None, "foreign_keys", true)
            .context("failed to enable foreign keys support")?;

        init_extract_refs_fn(&conn, self.schema.clone())?;
        init_calculate_search_score_fn(&conn, self.schema.clone())?;
        init_json_contains(&conn)?;

        Ok(conn)
    }

    fn get_connection(&self) -> Result<ArhivConnection> {
        let conn = self.open_connection(false)?;

        Ok(ArhivConnection::new(conn, self.path_manager.clone()))
    }

    pub fn get_tx(&self) -> Result<ArhivConnection> {
        let conn = self.open_connection(true)?;

        ArhivConnection::new_tx(conn, self.path_manager.clone(), self.schema.clone())
    }

    fn vacuum(&self) -> Result<()> {
        let now = Instant::now();

        let conn = self.open_connection(true)?;
        conn.execute("VACUUM", [])?;

        log::debug!(
            "completed VACUUM in {} seconds",
            now.elapsed().as_secs_f32()
        );

        Ok(())
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

    pub fn list_documents(&self, filter: impl AsRef<Filter>) -> Result<ListPage<Document>> {
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

fn json_contains(data: &str, field: &str, value: &str) -> Result<bool> {
    let data: Value = serde_json::from_str(data)?;

    let data = if let Some(data) = data.get(field) {
        data
    } else {
        return Ok(false);
    };

    if let Some(data) = data.as_str() {
        return Ok(data == value);
    }

    if let Some(data) = data.as_array() {
        let result = data
            .iter()
            .any(|item| item.as_str().map_or(false, |item| item == value));

        return Ok(result);
    }

    bail!("data must be string or array")
}

fn init_calculate_search_score_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let calculate_search_score = move |ctx: &FunctionContext| -> Result<usize> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let pattern = ctx.get_raw(2).as_str().context("pattern must be str")?;

        if pattern.is_empty() {
            return Ok(1);
        }

        let data_description = schema.get_data_description(document_type)?;
        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let result = data_description.search(&document_data, pattern);

        if let Err(ref err) = result {
            log::error!("calculate_search_score() failed: \n{}", err);
        }

        result
    };

    conn.create_scalar_function(
        "calculate_search_score",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            calculate_search_score(ctx)
                .context("calculate_search_score() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'calculate_search_score'")
}

fn init_json_contains(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "json_contains",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            let data = ctx.get_raw(0).as_str().expect("data must be str");
            let field = ctx.get_raw(1).as_str().expect("field must be str");
            let value = ctx.get_raw(2).as_str().expect("value must be str");

            json_contains(data, field, value)
                .context("json_contains() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'json_contains'")
}

fn init_extract_refs_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let extract_refs = move |ctx: &FunctionContext| -> Result<String> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let refs = schema.extract_refs(document_type, &document_data)?;

        serde_json::to_string(&refs).context("failed to serialize refs")
    };

    conn.create_scalar_function(
        "extract_refs",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

            let result = extract_refs(ctx);

            if let Err(ref err) = result {
                log::error!("extract_refs() failed: \n{:?}", err);
            }

            result.map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'extract_refs'")
}
