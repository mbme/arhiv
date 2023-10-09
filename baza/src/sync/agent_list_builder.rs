use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use reqwest::Url;
use tokio::time::timeout;

use rs_utils::{
    log,
    mdns::{MDNSService, PeerInfo},
};

use crate::{sync::BazaClient, Baza};

use super::SyncAgent;

pub struct AgentListBuilder {
    downloads_dir: String,
    agents: Vec<SyncAgent>,
}

impl AgentListBuilder {
    pub fn new(downloads_dir: String) -> Self {
        AgentListBuilder {
            agents: Vec::new(),
            downloads_dir,
        }
    }

    pub fn add_agent(&mut self, agent: SyncAgent) {
        self.agents.push(agent);
    }

    pub fn add_network_agent(&mut self, url: &str) -> Result<()> {
        let client = BazaClient::new(
            Url::from_str(url).context("failed to parse url")?,
            &self.downloads_dir,
        );

        self.add_agent(SyncAgent::new_in_network(client));

        Ok(())
    }

    pub fn add_in_mem_agent(&mut self, other_baza: Arc<Baza>) -> Result<()> {
        self.add_agent(SyncAgent::new_in_memory(other_baza)?);

        Ok(())
    }

    pub fn add_agents(&mut self, agents: impl IntoIterator<Item = SyncAgent>) {
        for agent in agents {
            self.add_agent(agent);
        }
    }

    pub fn parse_network_agents(&mut self, urls: &[String]) -> Result<()> {
        for url in urls {
            self.add_network_agent(url)?;
        }

        log::debug!("added {} network agents", urls.len());

        Ok(())
    }

    pub async fn discover_mdns_network_agents(
        &mut self,
        mdns_service: &MDNSService,
        peer_discovery_timeout: Duration,
    ) -> Result<usize> {
        log::info!("Collecting MDNS peers...");

        let rx = mdns_service.get_peers_rx();

        if let Ok(Ok(peers)) = timeout(
            peer_discovery_timeout,
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

            self.parse_network_agents(&urls)?;

            Ok(urls.len())
        } else {
            Ok(0)
        }
    }

    pub fn build(self) -> Vec<SyncAgent> {
        self.agents
    }
}
