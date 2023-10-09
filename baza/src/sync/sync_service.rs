use std::{str::FromStr, sync::Arc};

use anyhow::{ensure, Context, Result};
use reqwest::Url;

use rs_utils::{log, now};

use crate::{Baza, SETTING_LAST_SYNC_TIME};

use super::{agent::SyncAgent, ping::Ping, BazaClient};

pub struct SyncService<'b> {
    agents: Vec<Arc<SyncAgent>>,
    baza: &'b Baza,
}

impl<'b> SyncService<'b> {
    pub fn new(baza: &'b Baza) -> Self {
        Self {
            agents: Default::default(),
            baza,
        }
    }

    pub fn add_agent(&mut self, agent: SyncAgent) {
        self.agents.push(agent.into());
    }

    pub fn add_agents(&mut self, agents: impl IntoIterator<Item = SyncAgent>) {
        for agent in agents {
            self.add_agent(agent);
        }
    }

    pub fn parse_network_agents(&mut self, urls: &[String]) -> Result<()> {
        let agents = urls
            .iter()
            .map(|url| {
                let client = BazaClient::new(
                    Url::from_str(url).context("failed to parse url")?,
                    &self.baza.get_path_manager().downloads_dir,
                );

                Ok(SyncAgent::new_in_network(client))
            })
            .collect::<Result<Vec<_>>>()?;

        let count = agents.len();
        self.add_agents(agents);

        log::debug!("added {count} network agents");

        Ok(())
    }

    pub fn get_agents_count(&self) -> usize {
        self.agents.len()
    }

    async fn collect_pings(&self) -> Result<Vec<(Arc<SyncAgent>, Ping)>> {
        let ping = self.baza.get_connection()?.get_ping()?;

        let pings = self
            .agents
            .iter()
            .map(|agent| async { (agent.clone(), agent.exchange_pings(&ping).await) });

        let mut pings = futures::future::join_all(pings)
            .await
            .into_iter()
            .map(|(agent, ping_result)| Ok((agent, ping_result?)))
            .collect::<Result<Vec<_>>>()?;

        pings.sort_by_cached_key(|(_agent, ping)| ping.rev.clone());

        Ok(pings)
    }

    pub async fn sync(&self) -> Result<bool> {
        ensure!(
            !self.baza.get_connection()?.has_staged_documents()?,
            "There are uncommitted changes"
        );

        let pings = self.collect_pings().await?;

        log::info!("starting sync with {} other instances", pings.len());

        let mut updated = false;
        for (agent, ping) in pings {
            let local_rev = self.baza.get_connection()?.get_db_rev()?;

            if local_rev.is_concurrent_or_older_than(&ping.rev) {
                let changeset = agent.fetch_changes(&local_rev).await?;

                log::info!(
                    "applying changeset {} from {}",
                    &changeset,
                    ping.instance_id.as_ref()
                );

                let mut tx = self.baza.get_tx()?;

                let summary = tx.apply_changeset(changeset)?;

                // TODO parallel file download
                for (index, blob) in summary.missing_blobs.iter().enumerate() {
                    log::info!(
                        "downloading BLOB {} of {} from {}",
                        index + 1,
                        summary.missing_blobs.len(),
                        ping.instance_id.as_ref(),
                    );
                    agent.fetch_blob(blob).await?;
                }

                tx.kvs_const_set(SETTING_LAST_SYNC_TIME, &now())?;

                tx.commit()?;

                updated = updated || summary.has_changes();

                log::info!(
                    "got {} new snapshots and {} BLOBs from {}",
                    summary.new_snapshots,
                    summary.missing_blobs.len(),
                    ping.instance_id.as_ref()
                );
            } else {
                log::debug!(
                    "instance {} has same or older revision",
                    ping.instance_id.as_ref()
                );
            }
        }

        if updated {
            log::info!("finished sync, updated");
        } else {
            log::info!("finished sync, no updates");
        }

        Ok(updated)
    }
}
