use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use anyhow::{bail, Result};
use tokio::time::timeout;

use baza::{
    schema::{DataMigrations, DataSchema},
    sync::{build_rpc_router, SyncService},
    AutoCommitService, AutoCommitTask, Baza,
};
use rs_utils::{
    http_server::{build_health_router, check_server_health, HttpServer},
    log,
    mdns::{MDNSService, PeerInfo},
};

use crate::{
    config::Config, data_migrations::get_data_migrations, definitions::get_standard_schema,
    ui_server::build_ui_router,
};

const PEER_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(8);

pub struct Arhiv {
    pub baza: Arc<Baza>,
    pub(crate) config: Config,
    mdns_service: OnceLock<MDNSService>,
    auto_commit_task: Option<AutoCommitTask>,
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

        let baza = Arc::new(Baza::open(
            config.arhiv_root.clone(),
            schema,
            data_migrations,
        )?);

        let auto_commit_task = Arhiv::maybe_init_auto_commit_service(
            baza.clone(),
            config.auto_commit_delay_in_seconds,
        )?;

        Ok(Arhiv {
            baza,
            config,
            mdns_service: Default::default(),
            auto_commit_task,
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
        let baza = Arc::new(Baza::create(
            config.arhiv_root.clone(),
            schema,
            data_migrations,
        )?);

        let auto_commit_task = Arhiv::maybe_init_auto_commit_service(
            baza.clone(),
            config.auto_commit_delay_in_seconds,
        )?;

        Ok(Arhiv {
            baza,
            config,
            mdns_service: Default::default(),
            auto_commit_task,
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

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub async fn sync(&self) -> Result<bool> {
        log::info!("Starting arhiv sync");

        let mut sync_service = SyncService::new(&self.baza);

        let static_peers = &self.config.static_peers;
        if !static_peers.is_empty() {
            sync_service.parse_network_agents(static_peers)?;
            log::info!("Added {} static peers", static_peers.len());
        }

        let mdns_service = self.get_mdns_service();

        let rx = mdns_service.get_peers_rx();

        log::info!("Collecting MDNS peers...");
        if let Ok(Ok(peers)) = timeout(
            PEER_DISCOVERY_TIMEOUT,
            rx.clone().wait_for(|peers| !peers.is_empty()),
        )
        .await
        {
            let urls = peers
                .values()
                .flat_map(|PeerInfo { ips, port }| {
                    ips.iter()
                        .map(|ip| format!("http://{ip}:{port}"))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            sync_service.parse_network_agents(&urls)?;
            log::info!("Added {} MDNS peers", urls.len());
        } else {
            log::warn!(
                "MDNS service couldn't discover any peers in {}s",
                PEER_DISCOVERY_TIMEOUT.as_secs()
            )
        }

        if sync_service.get_agents_count() == 0 {
            bail!("no agents discovered");
        }

        sync_service.sync().await
    }

    fn get_mdns_service(&self) -> &MDNSService {
        self.mdns_service.get_or_init(|| {
            self.baza
                .init_mdns_service()
                .expect("must init MDNS service")
        })
    }

    pub async fn is_local_server_alive(&self) -> bool {
        let port = self.config.server_port;
        let local_server_url = format!("localhost:{port}");

        check_server_health(&local_server_url).await.is_ok()
    }

    pub fn stop(mut self) {
        if let Some(auto_commit_task) = self.auto_commit_task {
            auto_commit_task.abort();
        }

        if let Some(ref mut mdns_service) = self.mdns_service.get_mut() {
            mdns_service.shutdown();
        }
    }
}

pub async fn start_arhiv_server(arhiv: Arc<Arhiv>) -> Result<()> {
    let port = arhiv.config.server_port;

    let mdns_service = arhiv.get_mdns_service();
    let mut mdns_server = mdns_service.start_server(port)?;

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

    Ok(())
}
