use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};

use baza::{
    schema::{DataMigrations, DataSchema},
    sync::{build_rpc_router, AutoSyncTask, SyncManager},
    AutoCommitService, AutoCommitTask, Baza, BazaOptions,
};
use rs_utils::http_server::{build_health_router, check_server_health, HttpServer};

use crate::{
    config::Config, data_migrations::get_data_migrations, definitions::get_standard_schema,
    ui_server::build_ui_router,
};

const MDNS_PEER_DISCOVERY_DURATION: Duration = Duration::from_secs(8);

#[derive(Default)]
pub struct ArhivOptions {
    pub discover_peers: bool,
    pub auto_commit: bool,
}

pub struct Arhiv {
    pub baza: Arc<Baza>,
    config: Config,
    auto_commit_task: Option<AutoCommitTask>,
    auto_sync_task: Option<AutoSyncTask>,
    sync_manager: Arc<SyncManager>,
}

impl Arhiv {
    #[must_use]
    pub fn must_open() -> Arhiv {
        Arhiv::open_with_options(ArhivOptions::default()).expect("must be able to open arhiv")
    }

    pub fn open_with_options(options: ArhivOptions) -> Result<Arhiv> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        let baza = Baza::open(BazaOptions {
            root_dir: config.arhiv_root.clone(),
            schema,
            migrations: data_migrations,
        })?;
        let baza = Arc::new(baza);

        let mut sync_manager = SyncManager::new(baza.clone())?;
        if options.discover_peers {
            sync_manager.start_mdns_client(MDNS_PEER_DISCOVERY_DURATION)?;
        }

        let mut arhiv = Arhiv {
            baza,
            config,
            sync_manager: Arc::new(sync_manager),
            auto_commit_task: None,
            auto_sync_task: None,
        };
        if options.auto_commit {
            arhiv.maybe_init_auto_commit_service()?;
        }
        if options.discover_peers {
            arhiv.maybe_init_auto_sync_service()?;
        }

        Ok(arhiv)
    }

    pub fn create() -> Result<()> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        Arhiv::create_with_options(config, schema, data_migrations)
    }

    pub fn create_with_options(
        config: Config,
        schema: DataSchema,
        data_migrations: DataMigrations,
    ) -> Result<()> {
        Baza::create(BazaOptions {
            root_dir: config.arhiv_root.clone(),
            schema,
            migrations: data_migrations,
        })?;

        Ok(())
    }

    fn maybe_init_auto_commit_service(&mut self) -> Result<()> {
        let delay = self.config.auto_commit_delay_in_seconds;

        if delay > 0 {
            let service = AutoCommitService::new(self.baza.clone(), Duration::from_secs(delay));
            let task = service.start()?;

            self.auto_commit_task = Some(task);
        }

        Ok(())
    }

    fn maybe_init_auto_sync_service(&mut self) -> Result<()> {
        let delay = self.config.auto_sync_delay_in_seconds;

        if delay > 0 {
            let task = self
                .sync_manager
                .clone()
                .start_auto_sync(Duration::from_secs(delay))?;

            self.auto_sync_task = Some(task);
        }

        Ok(())
    }

    #[must_use]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub async fn sync(&self) -> Result<bool> {
        self.sync_manager.sync().await
    }

    #[must_use]
    pub async fn is_local_server_alive(&self) -> bool {
        let port = self.config.server_port;
        let local_server_url = format!("localhost:{port}");

        check_server_health(&local_server_url).await.is_ok()
    }

    pub async fn start_server(self) -> Result<()> {
        let port = self.config.server_port;

        let arhiv = Arc::new(self);
        {
            let mut mdns_server = arhiv.sync_manager.start_mdns_server(port)?;

            let health_router = build_health_router();
            let rpc_router = build_rpc_router();
            let ui_router = build_ui_router();

            let router = rpc_router
                .nest("/ui", ui_router.with_state(arhiv.clone()))
                .with_state(arhiv.baza.clone())
                .merge(health_router);

            let server = HttpServer::start(router, port);

            server.join().await?;

            mdns_server.stop();
        }

        Arc::into_inner(arhiv)
            .context("failed to unwrap Arhiv instance")?
            .stop();

        Ok(())
    }

    pub fn stop(self) {
        if let Some(auto_commit_task) = self.auto_commit_task {
            auto_commit_task.abort();
        }

        Arc::into_inner(self.sync_manager)
            .expect("failed to unwrap a SyncManager instance")
            .stop();
    }
}
