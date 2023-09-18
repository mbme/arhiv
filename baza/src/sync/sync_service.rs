use std::rc::Rc;

use anyhow::Result;

use rs_utils::{log, now};

use crate::{Baza, SETTING_LAST_SYNC_TIME};

use super::{agent::SyncAgent, ping::Ping};

pub struct SyncService<'b> {
    agents: Vec<Rc<SyncAgent>>,
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

    async fn collect_pings(&self) -> Result<Vec<(Rc<SyncAgent>, Ping)>> {
        let pings = self
            .agents
            .iter()
            .map(|agent| async { (agent.clone(), agent.fetch_ping().await) });

        let mut pings = futures::future::join_all(pings)
            .await
            .into_iter()
            .map(|(agent, ping_result)| Ok((agent, ping_result?)))
            .collect::<Result<Vec<_>>>()?;

        pings.sort_by_cached_key(|(_agent, ping)| ping.rev.clone());

        Ok(pings)
    }

    pub async fn sync(&self) -> Result<bool> {
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
