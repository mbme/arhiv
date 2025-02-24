use std::sync::Arc;

use anyhow::{ensure, Result};

use rs_utils::{crypto_key::CryptoKey, log, ExposeSecret, SecretString};

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
    pub const MIN_LOGIN_LENGTH: usize = 2;
    pub const MIN_PASSWORD_LENGTH: usize = CryptoKey::MIN_PASSWORD_LEN;

    pub fn new(login: String, password: SecretString) -> Result<Self> {
        ensure!(
            login.len() >= Self::MIN_LOGIN_LENGTH,
            "Login should be at least {} characters long",
            Self::MIN_LOGIN_LENGTH
        );
        ensure!(
            password.expose_secret().len() >= Self::MIN_PASSWORD_LENGTH,
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
}

impl Baza {
    fn new(root_dir: String, schema: DataSchema) -> Self {
        let path_manager = PathManager::new(root_dir);

        Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
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

        tx.set_data_version(baza.schema.get_latest_data_version())?;
        tx.set_instance_id(&InstanceId::generate())?;

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

    #[must_use]
    pub fn get_db(&self) -> DB {
        DB::new(self.path_manager.clone(), self.schema.clone())
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
}
