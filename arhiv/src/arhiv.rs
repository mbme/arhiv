use std::sync::Arc;

use anyhow::{ensure, Result};

use baza::{
    sync::{AutoSyncTask, MDNSClientTask, MDNSDiscoveryService, SyncManager},
    AutoCommitService, AutoCommitTask, Baza, BazaOptions,
};
use rs_utils::log;

use crate::{
    config::ArhivConfigExt, data_migrations::get_data_migrations, definitions::get_standard_schema,
    Status,
};

#[derive(Default)]
pub struct ArhivOptions {
    pub create: bool,
    pub discover_peers: bool,
    pub mdns_server: bool,
    pub auto_commit: bool,
}

pub struct Arhiv {
    pub baza: Arc<Baza>,
    auto_commit_task: Option<AutoCommitTask>,
    auto_sync_task: Option<AutoSyncTask>,
    mdns_client_task: Option<MDNSClientTask>,
    sync_manager: Arc<SyncManager>,
    mdns_discovery_service: MDNSDiscoveryService,
}

impl Arhiv {
    pub fn open(root_dir: impl Into<String>, options: ArhivOptions) -> Result<Arhiv> {
        let root_dir = root_dir.into();
        log::debug!("Arhiv root dir: {root_dir}");

        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        let baza = Baza::open(BazaOptions {
            create: options.create,
            root_dir,
            schema,
            migrations: data_migrations,
        })?;
        let baza = Arc::new(baza);

        let sync_manager = SyncManager::new(baza.clone());
        let sync_manager = Arc::new(sync_manager);

        let mdns_discovery_service = MDNSDiscoveryService::new(&baza)?;

        let mut arhiv = Arhiv {
            baza,
            sync_manager,
            auto_commit_task: None,
            auto_sync_task: None,
            mdns_client_task: None,
            mdns_discovery_service,
        };
        if options.auto_commit {
            arhiv.init_auto_commit_service()?;
        }
        if options.discover_peers {
            arhiv.init_auto_sync_service()?;
            arhiv.init_mdns_client_service()?;
        }
        if options.mdns_server {
            let port = arhiv.baza.get_connection()?.get_server_port()?;
            arhiv.mdns_discovery_service.start_mdns_server(port)?;
        }

        Ok(arhiv)
    }

    fn init_auto_commit_service(&mut self) -> Result<()> {
        let auto_commit_delay = self.baza.get_connection()?.get_auto_commit_delay()?;
        ensure!(
            !auto_commit_delay.is_zero(),
            "Config auto-commit delay must not be zero"
        );

        let service = AutoCommitService::new(self.baza.clone(), auto_commit_delay);
        let task = service.start()?;

        self.auto_commit_task = Some(task);

        Ok(())
    }

    fn init_auto_sync_service(&mut self) -> Result<()> {
        let auto_sync_delay = self.baza.get_connection()?.get_auto_sync_delay()?;
        ensure!(
            !auto_sync_delay.is_zero(),
            "Config auto-sync delay must not be zero"
        );

        let task = self.sync_manager.clone().start_auto_sync(auto_sync_delay)?;

        self.auto_sync_task = Some(task);

        Ok(())
    }

    fn init_mdns_client_service(&mut self) -> Result<()> {
        let task = self
            .mdns_discovery_service
            .start_mdns_client(self.sync_manager.clone())?;
        self.mdns_client_task = Some(task);

        Ok(())
    }

    pub async fn get_status(&self) -> Result<Status> {
        let conn = self.baza.get_connection()?;

        let mut status = Status::read(&conn)?;
        status.check_if_local_server_is_running().await?;

        Ok(status)
    }

    pub async fn sync(&self) -> Result<bool> {
        self.sync_manager.sync().await
    }

    pub fn has_sync_agents(&self) -> bool {
        self.sync_manager.count_agents() > 0
    }

    pub fn stop(mut self) {
        if let Some(mdns_client_task) = self.mdns_client_task.take() {
            mdns_client_task.abort();
        }

        if let Some(auto_commit_task) = self.auto_commit_task.take() {
            auto_commit_task.abort();
        }

        if let Some(auto_sync_task) = self.auto_sync_task.take() {
            auto_sync_task.abort();
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        log::info!("Stopped Arhiv");
    }
}
