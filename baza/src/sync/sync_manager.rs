use std::{cell::RefCell, sync::Arc};

use anyhow::Result;

use rs_utils::log;

use crate::Baza;

use super::{agent::SyncAgent, instance_id::InstanceId, ping::Ping};

pub struct SyncManager {
    agents: RefCell<Vec<SyncAgent>>,
    baza: Arc<Baza>,
}

impl SyncManager {
    pub fn new(baza: Arc<Baza>) -> Self {
        Self {
            agents: Default::default(),
            baza,
        }
    }

    pub fn add_agent(&self, agent: SyncAgent) {
        self.agents.borrow_mut().push(agent);
    }

    pub fn remove_agent(&self, id: &InstanceId) {
        self.agents
            .borrow_mut()
            .retain(|agent| agent.get_id() != id);
    }

    pub async fn refresh_peers(&self) -> Result<()> {
        let agents = self.agents.borrow();

        for agent in agents.iter() {
            // TODO parallel
            agent.fetch_ping().await?;
        }

        Ok(())
    }

    pub async fn sync(&self) -> Result<bool> {
        let agents = self.agents.borrow();
        let mut agents = agents
            .iter()
            .filter(|agent| agent.get_ping().is_some())
            .collect::<Vec<_>>();

        agents.sort_by_cached_key(|agent| agent.get_ping().expect("must have ping").rev);

        log::info!("starting sync with {} other instances", agents.len());

        let mut updated = false;

        for agent in agents.iter() {
            let local_rev = self.baza.get_connection()?.get_db_rev()?;
            let ping = agent.get_ping().expect("must have ping");

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
                    agent.fetch_blob(&blob).await?;
                }

                tx.commit()?;

                updated = updated || summary.has_changes();
            }
        }

        log::info!("finished sync");

        Ok(updated)
    }

    pub fn get_pings(&self) -> Vec<Ping> {
        self.agents
            .borrow()
            .iter()
            .filter_map(|agent| agent.get_ping())
            .collect()
    }
}
