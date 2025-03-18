use std::sync::Arc;

use anyhow::Result;

use baza::{baza2::BazaManager, AutoCommitService, AutoCommitTask, DEV_MODE};
use rs_utils::{
    get_linux_data_home, get_linux_downloads_dir, get_linux_home_dir, into_absolute_path, log,
};

use crate::{definitions::get_standard_schema, ServerInfo, Status};

use keyring::SystemKeyring;
pub use keyring::{Keyring, NoopKeyring};
pub use scaled_images_cache::ImageParams;
use scaled_images_cache::ScaledImagesCache;

mod import;
mod keyring;
mod scaled_images_cache;

pub struct ArhivOptions {
    pub storage_dir: String,
    pub state_dir: String,
    pub downloads_dir: String,
    pub file_browser_root_dir: String,
    pub keyring: Arc<dyn Keyring>,
}

impl ArhivOptions {
    pub fn new_desktop() -> Self {
        let home_dir = get_linux_home_dir().unwrap_or("/".to_string());
        let data_dir = get_linux_data_home();
        let downloads_dir = get_linux_downloads_dir();

        let file_browser_root_dir = home_dir.clone();

        let keyring: Arc<dyn Keyring> = if cfg!(test) {
            Arc::new(NoopKeyring)
        } else {
            Arc::new(SystemKeyring)
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
    pub img_cache: ScaledImagesCache,
    pub keyring: Arc<dyn Keyring>,

    auto_commit_task: Option<AutoCommitTask>,
    file_browser_root_dir: String,
}

impl Arhiv {
    pub fn new(options: ArhivOptions) -> Self {
        let schema = get_standard_schema();

        let img_cache_dir = format!("{}/img-cache", options.state_dir);

        let baza_manager = BazaManager::new(
            options.storage_dir,
            options.state_dir,
            options.downloads_dir,
            schema,
        );
        let baza_manager = Arc::new(baza_manager);

        let img_cache = ScaledImagesCache::new(img_cache_dir, baza_manager.clone());

        Arhiv {
            baza: baza_manager,
            img_cache,
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

    pub fn collect_server_info(&self) -> Result<Option<ServerInfo>> {
        ServerInfo::collect(self.baza.get_state_dir())
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
}
