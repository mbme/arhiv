use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use tokio::{sync::broadcast::Sender, task::JoinHandle};

use rs_utils::{log, now, ScheduledTask, SelfSignedCertificate};

use crate::{entities::InstanceId, Baza, BazaEvent};

use super::{Ping, SyncAgent};

pub type AutoSyncTask = JoinHandle<()>;
pub type MDNSClientTask = JoinHandle<()>;

pub struct SyncManager {
    baza: Arc<Baza>,
    agents: Arc<Mutex<Vec<SyncAgent>>>,
    sync_in_progress: Arc<AtomicBool>,
}

impl SyncManager {
    pub fn new(baza: Arc<Baza>) -> Self {
        SyncManager {
            baza,
            agents: Default::default(),
            sync_in_progress: Default::default(),
        }
    }

    fn add_agent(&self, agent: SyncAgent) -> Result<()> {
        self.agents.lock().expect("must lock").push(agent);

        self.baza.publish_event(BazaEvent::PeerDiscovered {})?;

        Ok(())
    }

    pub fn add_network_agent(
        &self,
        instance_id: InstanceId,
        url: &str,
        certificate: &SelfSignedCertificate,
    ) -> Result<()> {
        let agent =
            SyncAgent::new_in_network(instance_id.clone(), url, certificate, self.baza.clone())?;

        self.add_agent(agent)?;

        log::info!("Added network agent {instance_id} {url}");

        Ok(())
    }

    pub fn add_in_mem_agent(&mut self, other_baza: Arc<Baza>) -> Result<()> {
        let agent = SyncAgent::new_in_memory(other_baza)?;

        self.add_agent(agent)?;

        Ok(())
    }

    pub fn count_agents(&self) -> usize {
        self.agents.lock().expect("must lock").len()
    }

    pub fn remove_agent(&self, instance_id: &InstanceId) {
        self.agents
            .lock()
            .expect("must lock")
            .retain(|agent| agent.get_instance_id() != instance_id);

        log::info!("Removed network agent {instance_id}");
    }

    pub fn remove_all_agents(&self) {
        self.agents.lock().expect("must lock").clear();

        log::info!("Removed all network agents");
    }

    async fn collect_pings(&self, agents: Vec<SyncAgent>) -> Result<Vec<(SyncAgent, Ping)>> {
        let ping = self.baza.get_connection()?.get_ping()?;

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

    pub async fn sync(&self) -> Result<bool> {
        log::info!("Starting sync");

        if self
            .sync_in_progress
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            log::warn!("Sync already in progress");
            return Ok(false);
        }

        let _guard = SyncGuard::new(self.sync_in_progress.clone(), self.baza.get_events_sender());

        {
            let conn = self.baza.get_connection()?;

            if conn.has_staged_documents()? {
                log::warn!("There are uncommitted changes");
                return Ok(false);
            }

            let locks = conn.list_document_locks()?;
            if !locks.is_empty() {
                log::warn!("There are {} pending locks", locks.len());
                return Ok(false);
            }
        }

        let agents = self.agents.lock().expect("must lock").clone();
        if agents.is_empty() {
            log::warn!("No agents discovered");
            return Ok(false);
        }

        let pings = self.collect_pings(agents).await?;

        log::info!("Starting sync with {} other instances", pings.len());

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

                tx.set_last_sync_time(&now())?;

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

    pub fn start_auto_sync(self: Arc<Self>, auto_sync_delay: Duration) -> Result<AutoSyncTask> {
        let task = tokio::spawn(async move {
            let mut events = self.baza.get_events_channel();

            let scheduled_sync = ScheduledTask::new();

            loop {
                let sync_manager = self.clone();

                match events.recv().await {
                    Ok(BazaEvent::InstanceOutdated {})
                    | Ok(BazaEvent::DocumentsCommitted {})
                    | Ok(BazaEvent::PeerDiscovered {}) => {
                        scheduled_sync.schedule(auto_sync_delay, async move {
                            if let Err(err) = sync_manager.sync().await {
                                log::error!("Auto-sync failed: {err}");
                            }
                        });
                    }
                    Ok(BazaEvent::Synced {}) => {
                        scheduled_sync.cancel();
                    }
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Error while polling events: {err}");
                        break;
                    }
                }
            }

            log::debug!("Auto-sync task ended");
        });

        log::info!(
            "Started auto-sync service, auto-sync delay is {} seconds",
            auto_sync_delay.as_secs()
        );

        Ok(task)
    }
}

struct SyncGuard {
    sync_in_progress: Arc<AtomicBool>,
    baza_events: Sender<BazaEvent>,
}

impl SyncGuard {
    #[must_use]
    pub fn new(sync_in_progress: Arc<AtomicBool>, baza_events: Sender<BazaEvent>) -> Self {
        sync_in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

        SyncGuard {
            sync_in_progress,
            baza_events,
        }
    }

    pub fn release(&self) {
        if let Err(err) = self.baza_events.send(BazaEvent::Synced {}) {
            log::error!("Failed to send baza event: {err}");
        }

        self.sync_in_progress
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        self.release();
    }
}
