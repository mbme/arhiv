use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use anyhow::Result;

use rs_utils::log;

use crate::{
    entities::{BLOBId, BLOB},
    Baza,
};

use super::{
    changeset::Changeset, instance_id::InstanceId, network::SyncNetwork, ping::Ping, Revision,
};

pub struct SyncAgent<'n, N: SyncNetwork> {
    pub(crate) baza: Rc<Baza>,
    id: InstanceId,
    known_instances: RefCell<HashMap<InstanceId, Ping>>,
    network: &'n N,
    ping_timeout: Duration,
}

impl<'n, N: SyncNetwork> SyncAgent<'n, N> {
    pub fn new(baza: Rc<Baza>, network: &'n N) -> Result<Self> {
        let id = baza.get_connection()?.get_instance_id()?;

        Ok(SyncAgent {
            id,
            baza,
            network,
            known_instances: Default::default(),
            ping_timeout: Duration::from_secs(30),
        })
    }

    pub fn get_id(&self) -> &InstanceId {
        &self.id
    }

    pub(crate) fn handle_ping(&self, ping: Ping) -> Result<Option<Ping>> {
        log::debug!("Got {ping}");
        let local_ping = self.baza.get_connection()?.get_ping()?;

        let local_has_changes = local_ping.rev.is_concurrent_or_newer_than(&ping.rev);

        self.known_instances
            .borrow_mut()
            .insert(ping.instance_id.clone(), ping);

        Ok(local_has_changes.then_some(local_ping))
    }

    pub(crate) fn handle_changes_request(&self, min_rev: &Revision) -> Result<Changeset> {
        self.baza.get_connection()?.get_changeset(min_rev)
    }

    pub(crate) fn handle_blob_request(&self, id: &BLOBId) -> Result<Option<BLOB>> {
        let blob = self.baza.get_connection()?.get_blob(id);

        Ok(blob.exists()?.then_some(blob))
    }

    pub async fn refresh_peers(&self) -> Result<()> {
        let ping = self.baza.get_connection()?.get_ping()?;

        let ping_task = self.network.ping_all(&ping);

        if let Err(_) = tokio::time::timeout(self.ping_timeout, ping_task).await {
            log::warn!("ping_all timed out");
        }

        Ok(())
    }

    pub fn get_pings(&self) -> Result<Vec<Ping>> {
        let mut pings = self
            .known_instances
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();

        pings.sort_by_cached_key(|ping| ping.rev.clone());

        Ok(pings)
    }

    pub async fn sync(&self) -> Result<bool> {
        let pings = self.get_pings()?;

        log::info!("starting sync with {} other instances", pings.len());

        let mut updated = false;

        for ping in pings.iter().rev() {
            let local_rev = self.baza.get_connection()?.get_db_rev()?;

            if local_rev.is_concurrent_or_older_than(&ping.rev) {
                let changeset = self
                    .network
                    .pull_changes(&ping.instance_id, &local_rev)
                    .await?;

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
                    self.network.fetch_blob(&ping.instance_id, &blob).await?;
                }

                tx.commit()?;

                updated = updated || summary.has_changes();
            }
        }

        log::info!("finished sync");

        Ok(updated)
    }
}

impl<'n, N: SyncNetwork> PartialEq for SyncAgent<'n, N> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
