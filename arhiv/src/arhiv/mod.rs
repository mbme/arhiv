mod import;
mod keyring;
mod status;

use std::{cmp::min, sync::Arc};

use anyhow::{Result, bail};

use baza::{AutoCommitService, AutoCommitTask, BazaManager, BazaPaths, DEV_MODE};
use rs_utils::{
    SecretString, get_linux_data_home, get_linux_downloads_dir, get_linux_home_dir,
    into_absolute_path, log, num_cpus,
};

use crate::definitions::get_standard_schema;

pub use self::keyring::{ArhivKeyring, Keyring};
pub use self::status::Status;

pub struct ArhivOptions {
    pub storage_dir: String,
    pub state_dir: String,
    pub downloads_dir: String,
    pub file_browser_root_dir: String,
    pub keyring: ArhivKeyring,
}

impl ArhivOptions {
    pub fn new_desktop() -> Self {
        let home_dir = get_linux_home_dir().unwrap_or("/".to_string());
        let data_dir = get_linux_data_home();
        let downloads_dir = get_linux_downloads_dir();

        let file_browser_root_dir = home_dir.clone();

        let keyring = if cfg!(test) {
            ArhivKeyring::new_noop()
        } else {
            ArhivKeyring::new_system_keyring()
        };

        if DEV_MODE {
            let dev_root =
                std::env::var("DEV_ARHIV_ROOT").expect("env variable DEV_ARHIV_ROOT is missing");

            let dev_root = into_absolute_path(dev_root, false)
                .expect("can't turn DEV_ARHIV_ROOT into absolute path");

            return ArhivOptions {
                storage_dir: format!("{dev_root}/storage"),
                state_dir: format!("{dev_root}/state"),
                downloads_dir: format!("{dev_root}/downloads"),
                file_browser_root_dir,
                keyring,
            };
        }

        let storage_dir = format!("{home_dir}/arhiv");
        let state_dir = format!("{}/arhiv-state", data_dir.unwrap_or(home_dir.clone()));
        let downloads_dir = downloads_dir.unwrap_or(format!("{home_dir}/Downloads"));

        ArhivOptions {
            storage_dir,
            state_dir,
            downloads_dir,
            file_browser_root_dir,
            keyring,
        }
    }
}

pub struct Arhiv {
    pub baza: Arc<BazaManager>,
    keyring: ArhivKeyring,
    auto_commit_task: Option<AutoCommitTask>,
    file_browser_root_dir: String,
}

impl Arhiv {
    pub fn new(options: ArhivOptions) -> Self {
        let schema = get_standard_schema();

        let paths = BazaPaths::new(
            options.storage_dir,
            options.state_dir,
            options.downloads_dir,
        );
        let baza_manager = BazaManager::new(paths, schema);
        let baza_manager = Arc::new(baza_manager);

        Arhiv {
            baza: baza_manager,
            keyring: options.keyring,

            auto_commit_task: None,
            file_browser_root_dir: options.file_browser_root_dir,
        }
    }

    pub fn new_desktop() -> Self {
        Arhiv::new(ArhivOptions::new_desktop())
    }

    pub fn init_auto_commit_service(&mut self) {
        let auto_commit_delay = AutoCommitService::DEFAULT_AUTO_COMMIT_DELAY;
        if auto_commit_delay.is_zero() {
            panic!("Config auto-commit delay must not be zero");
        }

        let service = AutoCommitService::new(self.baza.clone(), auto_commit_delay);
        let task = service.start();

        self.auto_commit_task = Some(task);
    }

    pub fn create(&self, password: SecretString) -> Result<()> {
        log::info!("Creating new Arhiv");

        if self.baza.storage_exists()? {
            bail!("Arhiv already exists");
        }

        self.baza.create(password.clone())?;
        self.keyring.set_password(Some(password))?;

        Ok(())
    }

    pub fn lock(&self) -> Result<()> {
        log::info!("Locking Arhiv");

        self.baza.lock()?;
        self.keyring.set_password(None)?;

        Ok(())
    }

    pub fn unlock(&self, password: SecretString) -> Result<()> {
        log::info!("Unlocking Arhiv");

        self.baza.unlock(password.clone())?;
        self.keyring.set_password(Some(password))?;

        Ok(())
    }

    pub fn unlock_using_keyring(&self) -> Result<bool> {
        let password = self.keyring.get_password()?;

        if let Some(password) = password {
            self.baza.unlock(password)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn change_password(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<()> {
        log::info!("Changing Arhiv password");

        self.baza
            .change_key_file_password(old_password, new_password.clone())?;

        self.keyring.set_password(Some(new_password))?;

        Ok(())
    }

    pub fn get_status(&self) -> Result<String> {
        let conn = self.baza.open()?;

        let status = Status::read(&conn)?;

        Ok(status.to_string())
    }

    pub fn get_file_browser_root_dir(&self) -> &str {
        &self.file_browser_root_dir
    }

    pub fn stop(&self) {
        if let Some(ref auto_commit_task) = self.auto_commit_task {
            auto_commit_task.abort();
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        log::info!("Stopped Arhiv");
    }

    pub fn optimal_number_of_worker_threads() -> usize {
        let num_cpus = num_cpus().ok().unwrap_or(1);

        min(num_cpus, 3)
    }
}
