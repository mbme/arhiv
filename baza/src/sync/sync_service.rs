use std::{collections::HashMap, sync::Arc};

use anyhow::Result;

use rs_utils::log;

use crate::Baza;

use super::{agent::SyncAgent, instance_id::InstanceId, ping::Ping};

pub struct SyncService {
    agents: HashMap<InstanceId, SyncAgent>,
    baza: Arc<Baza>,
}

impl SyncService {
    pub fn new(baza: Arc<Baza>) -> Self {
        Self {
            agents: Default::default(),
            baza,
        }
    }

    pub fn add_agent(&mut self, agent: SyncAgent) {
        self.agents.insert(agent.get_id().clone(), agent);
    }

    pub fn remove_agent(&mut self, id: &InstanceId) {
        self.agents.remove(id);
    }

    async fn collect_pings(&self) -> Result<Vec<Ping>> {
        let pings = self.agents.values().map(|agent| agent.fetch_ping());

        let mut pings = futures::future::join_all(pings)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        pings.sort_by_cached_key(|ping| ping.rev.clone());

        Ok(pings)
    }

    pub async fn sync(&self) -> Result<bool> {
        let pings = self.collect_pings().await?;

        log::info!("starting sync with {} other instances", pings.len());

        let mut updated = false;
        for ping in pings {
            let agent = self
                .agents
                .get(&ping.instance_id)
                .expect("agent is missing");

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

                tx.commit()?;

                updated = updated || summary.has_changes();
            } else {
                log::debug!(
                    "instance {} has same or older revision",
                    ping.instance_id.as_ref()
                );
            }
        }

        log::info!("finished sync");

        Ok(updated)
    }
}
