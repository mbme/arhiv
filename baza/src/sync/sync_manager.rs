use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{ensure, Context, Result};

use rs_utils::{
    log,
    mdns::{MDNSEvent, MDNSServer, MDNSService},
    now,
};
use tokio::time::{sleep_until, Instant};

use crate::{Baza, BazaEvent, DEBUG_MODE, SETTING_LAST_SYNC_TIME};

use super::{InstanceId, Ping, SyncAgent};

pub struct SyncManager {
    baza: Arc<Baza>,
    mdns_service: MDNSService,
    agents: Arc<Mutex<Vec<SyncAgent>>>,
    mdns_client_discovery_complete: Option<Instant>,
}

impl SyncManager {
    pub fn new(baza: Arc<Baza>) -> Result<Self> {
        let instance_id = baza
            .get_connection()
            .and_then(|conn| conn.get_instance_id())
            .context("failed to read instance_id")?;

        let app_name = baza.get_name();

        let mut service_name = format!("_{app_name}-baza");
        if DEBUG_MODE {
            service_name.push_str("-debug");
        }

        let mdns_service = MDNSService::new(service_name, instance_id)?;

        Ok(SyncManager {
            baza,
            mdns_service,
            mdns_client_discovery_complete: None,
            agents: Default::default(),
        })
    }

    pub fn start_mdns_client(&mut self, initial_discovery_duration: Duration) -> Result<()> {
        if self.mdns_client_discovery_complete.is_some() {
            log::warn!("MDNS client already started");
            return Ok(());
        }

        self.mdns_client_discovery_complete = Some(Instant::now() + initial_discovery_duration);

        self.mdns_service.start_client()?;

        let mut mdns_events = self.mdns_service.get_events();
        let baza_events = self.baza.get_events_sender();
        let agents = self.agents.clone();
        let downloads_dir = self.baza.get_path_manager().downloads_dir.clone();
        tokio::spawn(async move {
            match mdns_events.recv().await {
                Ok(mdns_event) => match mdns_event {
                    MDNSEvent::InstanceDiscovered(peer_info) => {
                        let instance_id = InstanceId::from_string(peer_info.instance_name);
                        let address = format!("http://{}:{}", peer_info.ips[0], peer_info.port);

                        match SyncAgent::new_in_network(
                            instance_id.clone(),
                            &address,
                            &downloads_dir,
                        ) {
                            Ok(agent) => {
                                agents.lock().expect("must lock").push(agent);
                                log::debug!("Added network agent {instance_id} {address}");

                                baza_events
                                    .send(BazaEvent::PeerDiscovered {})
                                    .expect("failed to publish baza event");
                            }
                            Err(err) => {
                                log::error!(
                                    "Failed to add network agent {instance_id} {address}: {err}"
                                );
                            }
                        }
                    }
                    MDNSEvent::InstanceDisappeared(instance_name) => {
                        let instance_id = InstanceId::from_string(instance_name);

                        agents
                            .lock()
                            .expect("must lock")
                            .retain(|agent| agent.get_instance_id() != &instance_id);

                        log::debug!("Removed network agent {instance_id}");
                    }
                },
                Err(err) => log::error!("Failed to receive MDNS event: {err}"),
            }
        });

        Ok(())
    }

    pub fn start_mdns_server(&self, port: u16) -> Result<MDNSServer> {
        self.mdns_service.start_server(port)
    }

    fn add_agent(&mut self, agent: SyncAgent) -> Result<()> {
        self.agents.lock().expect("must lock").push(agent);

        self.baza.publish_event(BazaEvent::PeerDiscovered {})?;

        Ok(())
    }

    pub fn add_network_agent(&mut self, instance_id: InstanceId, url: &str) -> Result<()> {
        let agent = SyncAgent::new_in_network(
            instance_id,
            url,
            &self.baza.get_path_manager().downloads_dir,
        )?;

        self.add_agent(agent)?;

        Ok(())
    }

    pub fn add_in_mem_agent(&mut self, other_baza: Arc<Baza>) -> Result<()> {
        let agent = SyncAgent::new_in_memory(other_baza)?;

        self.add_agent(agent)?;

        Ok(())
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

        ensure!(
            !self.baza.get_connection()?.has_staged_documents()?,
            "There are uncommitted changes"
        );

        if let Some(mdns_client_discovery_complete) = self.mdns_client_discovery_complete {
            let time_left = mdns_client_discovery_complete - Instant::now();
            if time_left.as_millis() > 0 {
                log::info!(
                    "waiting {}s until initial MDNS client discovery is complete",
                    time_left.as_secs()
                );
                sleep_until(mdns_client_discovery_complete).await;
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

    pub fn stop(mut self) {
        if self.mdns_client_discovery_complete.is_some() {
            self.mdns_service.stop_client();
        }

        self.mdns_service.shutdown();
    }
}
