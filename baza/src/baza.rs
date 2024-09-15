use std::sync::{Arc, OnceLock};

use anyhow::{ensure, Context, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};

use rs_utils::{log, CryptoKey, SecretString, MIN_TIMESTAMP};

pub use crate::events::BazaEvent;
use crate::{
    db::BazaConnection, entities::InstanceId, path_manager::PathManager, schema::DataSchema,
    DocumentExpert, DB,
};

pub struct BazaOptions {
    pub root_dir: String,
    pub schema: DataSchema,
}

pub struct Credentials {
    login: String,
    password: SecretString,
}

impl Credentials {
    pub const MIN_LOGIN_LENGTH: usize = 3;
    pub const MIN_PASSWORD_LENGTH: usize = CryptoKey::MIN_PASSWORD_LENGTH;

    pub fn new(login: impl Into<String>, password: impl Into<SecretString>) -> Result<Self> {
        let login = login.into();
        let password = password.into();

        ensure!(
            login.len() >= Self::MIN_LOGIN_LENGTH,
            "Login should be at least {} characters long",
            Self::MIN_LOGIN_LENGTH
        );
        ensure!(
            password.len() >= Self::MIN_PASSWORD_LENGTH,
            "Password should be at least {} characters long",
            Self::MIN_PASSWORD_LENGTH
        );

        Ok(Credentials { login, password })
    }

    pub fn get_login(&self) -> &str {
        &self.login
    }
}

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    events: (Sender<BazaEvent>, Receiver<BazaEvent>),
    key: OnceLock<CryptoKey>,
}

impl Baza {
    fn new(root_dir: String, schema: DataSchema) -> Self {
        let path_manager = PathManager::new(root_dir);

        Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            events: channel(42),
            key: Default::default(),
        }
    }

    pub fn create(options: BazaOptions, auth: Credentials) -> Result<Baza> {
        let baza = Baza::new(options.root_dir, options.schema);

        log::info!(
            "Initializing {} Baza in {}",
            baza.get_app_name(),
            baza.path_manager.root_dir
        );

        baza.get_db().create()?;

        let tx = baza.get_tx()?;

        tx.set_schema_name(&baza.schema.get_app_name().to_string())?;
        tx.set_data_version(baza.schema.get_latest_data_version())?;
        tx.set_instance_id(&InstanceId::new())?;
        tx.set_last_sync_time(&MIN_TIMESTAMP)?;

        tx.commit()?;

        baza.update_credentials(auth)?;

        Ok(baza)
    }

    pub fn update_credentials(&self, auth: Credentials) -> Result<()> {
        let tx = self.get_tx()?;

        tx.set_login(&auth.login)?;
        tx.set_password(auth.password)?;

        tx.commit()?;

        log::debug!("Updated login & password");

        Ok(())
    }

    pub fn open(options: BazaOptions) -> Result<Baza> {
        let baza = Baza::new(options.root_dir, options.schema);

        let schema_name = baza.get_connection()?.get_schema_name()?;
        let new_schema_name = baza.schema.get_app_name();
        ensure!(
            new_schema_name == schema_name,
            "Expected schema name to be '{schema_name}', but got '{new_schema_name}'"
        );

        baza.path_manager.assert_dirs_exist()?;
        baza.path_manager.assert_db_file_exists()?;

        let db = baza.get_db();

        db.apply_db_migrations()?;
        db.apply_data_migrations()?;

        log::debug!(
            "Opened {} Baza in {}",
            baza.get_app_name(),
            baza.path_manager.root_dir
        );

        Ok(baza)
    }

    fn create_key(&self) -> Result<CryptoKey> {
        let conn = self.get_connection()?;

        let app_name = self.get_app_name();
        let login = conn.get_login()?;
        let password = conn.get_password()?.into();

        CryptoKey::derive_from_password_with_argon2(&password, format!("{login}@{app_name}"))
    }

    pub fn get_key(&self) -> &CryptoKey {
        self.key.get_or_init(|| {
            self.create_key()
                .expect("Baza crypto key must be created successfully")
        })
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
