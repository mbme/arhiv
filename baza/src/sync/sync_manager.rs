use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use tokio::{task::JoinHandle, time::interval};

use rs_utils::{log, now};

use crate::{entities::InstanceId, Baza};

use super::SyncAgent;

pub type AutoSyncTask = JoinHandle<()>;
pub type MDNSClientTask = JoinHandle<()>;

pub struct SyncManager {
    baza: Arc<Baza>,
    agents: Arc<Mutex<HashMap<InstanceId, SyncAgent>>>,
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

    fn add_agent(&self, new_agent: SyncAgent) -> Result<()> {
        self.agents
            .lock()
            .expect("must lock")
            .insert(new_agent.get_instance_id().clone(), new_agent);

        Ok(())
    }

    pub fn add_network_agent(&self, instance_id: InstanceId, url: &str) -> Result<()> {
        let agent = SyncAgent::new_in_network(instance_id.clone(), url, self.baza.clone())?;

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
        let removed = self
            .agents
            .lock()
            .expect("must lock")
            .remove(instance_id)
            .is_some();

        if removed {
            log::info!("Removed sync agent {instance_id}");
        } else {
            log::warn!("Couldn't remove sync agent {instance_id}: not found");
        }
    }

    pub fn remove_all_agents(&self) {
        self.agents.lock().expect("must lock").clear();

        log::info!("Removed all network agents");
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

        let _guard = SyncGuard::new(self.sync_in_progress.clone());

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

        log::info!("Starting sync with {} other instances", agents.len());

        let mut updated = false;
        for (instance_id, agent) in agents {
            let request = self.baza.get_connection()?.get_changeset_request()?;

            let changeset = match agent.fetch_changes(&request).await {
                Ok(changeset) => changeset,
                Err(err) => {
                    log::warn!("Failed to fetch changes from agent {instance_id}: {err}");
                    continue;
                }
            };

            log::info!("Applying changeset {changeset} from {instance_id}");

            let mut tx = self.baza.get_tx()?;

            let summary = tx.apply_changeset(changeset)?;

            // TODO parallel file download
            for (index, blob) in summary.missing_blobs.iter().enumerate() {
                log::info!(
                    "Downloading BLOB {} of {} from {instance_id}",
                    index + 1,
                    summary.missing_blobs.len(),
                );
                agent.fetch_blob(blob).await?;
            }

            tx.set_last_sync_time(&now())?;

            tx.commit()?;

            updated = updated || summary.has_changes();

            log::info!(
                "Got {} new snapshots and {} BLOBs from {}",
                summary.new_snapshots,
                summary.missing_blobs.len(),
                &instance_id
            );
        }

        if updated {
            log::info!("Finished sync, updated");
        } else {
            log::info!("Finished sync, no updates");
        }

        Ok(updated)
    }

    pub fn start_auto_sync(self: Arc<Self>, auto_sync_interval: Duration) -> Result<AutoSyncTask> {
        let task = tokio::spawn(async move {
            let mut interval = interval(auto_sync_interval / 2);

            loop {
                interval.tick().await;

                if let Err(err) = self.sync().await {
                    log::warn!("Auto-sync failed: {err}");
                    break;
                }
            }

            log::debug!("Auto-sync task ended");
        });

        log::info!(
            "Started auto-sync service, auto-sync interval is {} seconds",
            auto_sync_interval.as_secs()
        );

        Ok(task)
    }
}

struct SyncGuard {
    sync_in_progress: Arc<AtomicBool>,
}

impl SyncGuard {
    #[must_use]
    pub fn new(sync_in_progress: Arc<AtomicBool>) -> Self {
        sync_in_progress.store(true, std::sync::atomic::Ordering::SeqCst);

        SyncGuard { sync_in_progress }
    }

    pub fn release(&self) {
        self.sync_in_progress
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        self.release();
    }
}
