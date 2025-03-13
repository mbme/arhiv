use std::sync::Arc;

use anyhow::Result;

use baza::{baza2::BazaManager, AutoCommitService, AutoCommitTask, DEV_MODE};
use rs_utils::{get_data_home, get_home_dir, into_absolute_path, log};

use crate::{definitions::get_standard_schema, ServerInfo, Status};

pub use scaled_images_cache::ImageParams;
use scaled_images_cache::ScaledImagesCache;

mod import;
mod keyring;
mod scaled_images_cache;

#[derive(Clone)]
pub struct ArhivOptions {
    pub storage_dir: String,
    pub state_dir: String,
    pub file_browser_root_dir: String,
}

impl ArhivOptions {
    pub fn new_desktop() -> Self {
        let home_dir = get_home_dir();
        let data_dir = get_data_home();

        let file_browser_root_dir = home_dir.clone().unwrap_or("/".to_string());

        if DEV_MODE {
            let dev_root =
                std::env::var("DEV_ARHIV_ROOT").expect("env variable DEV_ARHIV_ROOT is missing");

            let dev_root = into_absolute_path(dev_root, false)
                .expect("can't turn DEV_ARHIV_ROOT into absolute path");

            return ArhivOptions {
                storage_dir: format!("{dev_root}/storage"),
                state_dir: format!("{dev_root}/state"),
                file_browser_root_dir,
            };
        }

        let storage_dir = home_dir
            .as_ref()
            .map_or("/arhiv-storage".to_string(), |home_dir| {
                format!("{home_dir}/arhiv")
            });
        let state_dir = data_dir.map_or("/arhiv-state".to_string(), |data_dir| {
            format!("{data_dir}/arhiv-state")
        });

        ArhivOptions {
            storage_dir,
            state_dir,
            file_browser_root_dir,
        }
    }

    pub fn new_android(app_files_dir: String, external_storage_dir: String) -> Self {
        let storage_dir = format!("{external_storage_dir}/Arhiv");

        ArhivOptions {
            storage_dir,
            state_dir: app_files_dir,
            file_browser_root_dir: external_storage_dir,
        }
    }
}

pub struct Arhiv {
    pub baza: Arc<BazaManager>,
    pub img_cache: ScaledImagesCache,

    auto_commit_task: Option<AutoCommitTask>,
    file_browser_root_dir: String,
}

impl Arhiv {
    pub fn new(options: ArhivOptions) -> Self {
        let schema = get_standard_schema();

        let img_cache_dir = format!("{}/img-cache", options.state_dir);

        let baza_manager = BazaManager::new(options.storage_dir, options.state_dir, schema);
        let baza_manager = Arc::new(baza_manager);

        let img_cache = ScaledImagesCache::new(img_cache_dir, baza_manager.clone());

        Arhiv {
            baza: baza_manager,
            img_cache,

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
