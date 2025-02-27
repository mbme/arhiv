use std::sync::Arc;

use anyhow::{ensure, Result};

use baza::{baza2::BazaManager, AutoCommitService, AutoCommitTask};
use rs_utils::{get_home_dir, log};

use crate::{definitions::get_standard_schema, Status};

#[derive(Default, Clone)]
pub struct ArhivOptions {
    pub auto_commit: bool,
    pub file_browser_root_dir: Option<String>,
    pub storage_dir: String,
    pub state_dir: String,
}

pub struct Arhiv {
    pub baza: Arc<BazaManager>,
    auto_commit_task: Option<AutoCommitTask>,
    file_browser_root_dir: String,
}

impl Arhiv {
    pub fn new(options: ArhivOptions) -> Self {
        let schema = get_standard_schema();

        let baza_manager = BazaManager::new(options.storage_dir, options.state_dir, schema);

        Arhiv {
            baza: Arc::new(baza_manager),
            auto_commit_task: None,
            file_browser_root_dir: options
                .file_browser_root_dir
                .or_else(get_home_dir)
                .unwrap_or_else(|| "/".to_string()),
        }
    }

    fn init_auto_commit_service(&mut self) -> Result<()> {
        let auto_commit_delay = AutoCommitService::DEFAULT_AUTO_COMMIT_DELAY;
        ensure!(
            !auto_commit_delay.is_zero(),
            "Config auto-commit delay must not be zero"
        );

        let service = AutoCommitService::new(self.baza.clone(), auto_commit_delay);
        let task = service.start()?;

        self.auto_commit_task = Some(task);

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
}
