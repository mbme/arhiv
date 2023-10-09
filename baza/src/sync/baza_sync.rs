use anyhow::{ensure, Context, Result};

use rs_utils::{log, mdns::MDNSService, now};

use crate::{Baza, DEBUG_MODE, SETTING_LAST_SYNC_TIME};

use super::{agent::SyncAgent, ping::Ping, AgentListBuilder};

impl Baza {
    pub fn init_mdns_service(&self) -> Result<MDNSService> {
        let instance_id = self
            .get_connection()
            .and_then(|conn| conn.get_instance_id())
            .context("failed to read instance_id")?;

        let app_name = self.get_name();

        let mut service_name = format!("_{app_name}-baza");
        if DEBUG_MODE {
            service_name.push_str("-debug");
        }

        MDNSService::new(service_name, instance_id)
    }

    pub fn new_agent_list_builder(&self) -> AgentListBuilder {
        AgentListBuilder::new(self.get_path_manager().downloads_dir.clone())
    }

    async fn collect_pings(&self, agents: Vec<SyncAgent>) -> Result<Vec<(SyncAgent, Ping)>> {
        let ping = self.get_connection()?.get_ping()?;

        let pings = agents.into_iter().map(|agent| async {
            let ping = agent.exchange_pings(&ping).await;
            (agent, ping)
        });

        let mut pings = futures::future::join_all(pings)
            .await
            .into_iter()
            .filter_map(|(agent, ping_result)| match ping_result {
                Ok(ping) => Some((agent, ping)),
                Err(err) => {
                    log::warn!("Failed to exchange pings with agent {agent}: {err}");

                    None
                }
            })
            .collect::<Vec<_>>();

        pings.sort_by_cached_key(|(_agent, ping)| ping.rev.clone());

        Ok(pings)
    }

    pub async fn sync(&self, agents: Vec<SyncAgent>) -> Result<bool> {
        ensure!(
            !self.get_connection()?.has_staged_documents()?,
            "There are uncommitted changes"
        );

        let pings = self.collect_pings(agents).await?;

        log::info!("Starting sync with {} other instances", pings.len());

        let mut updated = false;
        for (agent, ping) in pings {
            let local_rev = self.get_connection()?.get_db_rev()?;

            if local_rev.is_concurrent_or_older_than(&ping.rev) {
                let changeset = agent.fetch_changes(&local_rev).await?;

                log::info!(
                    "applying changeset {} from {}",
                    &changeset,
                    ping.instance_id.as_ref()
                );

                let mut tx = self.get_tx()?;

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
            log::info!("Finished sync, updated");
        } else {
            log::info!("Finished sync, no updates");
        }

        Ok(updated)
    }
}
