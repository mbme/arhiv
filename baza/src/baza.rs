use std::sync::{Arc, OnceLock};

use anyhow::{ensure, Result};

use rs_utils::{crypto_key::CryptoKey, log, SecretString};

use crate::{
    db::BazaConnection, entities::InstanceId, path_manager::PathManager, schema::DataSchema,
    DocumentExpert, DB,
};

pub struct BazaOptions {
    pub root_dir: String,
    pub schema: DataSchema,
}

pub struct Baza {
    path_manager: Arc<PathManager>,
    schema: Arc<DataSchema>,
    key: OnceLock<CryptoKey>,
}

impl Baza {
    pub const MIN_PASSWORD_LENGTH: usize = CryptoKey::MIN_PASSWORD_LEN;

    fn new(root_dir: String, schema: DataSchema) -> Self {
        let path_manager = PathManager::new(root_dir);

        Baza {
            path_manager: Arc::new(path_manager),
            schema: Arc::new(schema),
            key: Default::default(),
        }
    }

    pub fn create(options: BazaOptions, password: impl Into<SecretString>) -> Result<Baza> {
        let password = password.into();

        ensure!(
            password.len() >= Self::MIN_PASSWORD_LENGTH,
            "Password should be at least {} characters long",
            Self::MIN_PASSWORD_LENGTH
        );

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

        baza.update_password(password)?;

        Ok(baza)
    }

    pub fn update_password(&self, password: impl Into<SecretString>) -> Result<()> {
        let password = password.into();

        ensure!(
            password.len() >= Self::MIN_PASSWORD_LENGTH,
            "Password should be at least {} characters long",
            Self::MIN_PASSWORD_LENGTH
        );

        let tx = self.get_tx()?;

        tx.set_password(password)?;

        tx.commit()?;

        log::debug!("Updated password");

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

    fn create_shared_key(&self) -> Result<CryptoKey> {
        let conn = self.get_connection()?;

        let app_name = self.get_app_name();
        let password = conn.get_password()?.into();

        CryptoKey::derive_from_password_with_scrypt(
            &password,
            CryptoKey::salt_from_data(format!("{app_name} shared key"))?,
        )
    }

    pub fn get_shared_key(&self) -> &CryptoKey {
        self.key.get_or_init(|| {
            self.create_shared_key()
                .expect("Baza crypto key must be created successfully")
        })
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
