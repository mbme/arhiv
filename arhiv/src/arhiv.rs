use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};

use baza::{
    schema::{DataMigrations, DataSchema},
    sync::{build_rpc_router, SyncManager},
    AutoCommitService, AutoCommitTask, Baza, BazaOptions,
};
use rs_utils::http_server::{build_health_router, check_server_health, HttpServer};

use crate::{
    config::Config, data_migrations::get_data_migrations, definitions::get_standard_schema,
    ui_server::build_ui_router,
};

const MDNS_PEER_DISCOVERY_DURATION: Duration = Duration::from_secs(8);

pub struct Arhiv {
    pub baza: Arc<Baza>,
    pub(crate) config: Config,
    auto_commit_task: Option<AutoCommitTask>,
    sync_manager: SyncManager,
}

impl Arhiv {
    #[must_use]
    pub fn must_open() -> Arhiv {
        Arhiv::open().expect("must be able to open arhiv")
    }

    pub fn open() -> Result<Arhiv> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        let baza = Baza::open(BazaOptions {
            root_dir: config.arhiv_root.clone(),
            schema,
            migrations: data_migrations,
        })?;
        let baza = Arc::new(baza);

        let auto_commit_task = Arhiv::maybe_init_auto_commit_service(
            baza.clone(),
            config.auto_commit_delay_in_seconds,
        )?;

        let sync_manager = SyncManager::new(baza.clone())?;

        Ok(Arhiv {
            baza,
            config,
            auto_commit_task,
            sync_manager,
        })
    }

    pub fn create() -> Result<Self> {
        let config = Config::read()?.0;
        let schema = get_standard_schema();
        let data_migrations = get_data_migrations();

        Arhiv::create_with_options(config, schema, data_migrations)
    }

    pub fn create_with_options(
        config: Config,
        schema: DataSchema,
        data_migrations: DataMigrations,
    ) -> Result<Self> {
        let baza = Baza::create(BazaOptions {
            root_dir: config.arhiv_root.clone(),
            schema,
            migrations: data_migrations,
        })?;
        let baza = Arc::new(baza);

        let auto_commit_task = Arhiv::maybe_init_auto_commit_service(
            baza.clone(),
            config.auto_commit_delay_in_seconds,
        )?;

        let sync_manager = SyncManager::new(baza.clone())?;

        Ok(Arhiv {
            baza,
            config,
            auto_commit_task,
            sync_manager,
        })
    }

    fn maybe_init_auto_commit_service(
        baza: Arc<Baza>,
        delay: u64,
    ) -> Result<Option<AutoCommitTask>> {
        if delay > 0 {
            let service = AutoCommitService::new(baza.clone(), Duration::from_secs(delay));
            let task = service.start()?;

            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn start_mdns_client(&mut self) -> Result<()> {
        self.sync_manager
            .start_mdns_client(MDNS_PEER_DISCOVERY_DURATION)?;

        Ok(())
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

        self.sync_manager.stop();
    }
}
