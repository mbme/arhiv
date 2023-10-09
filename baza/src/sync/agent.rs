use std::{fmt::Display, sync::Arc};

use anyhow::{Context, Result};

use crate::{entities::BLOB, Baza, BazaEvent};

use super::{changeset::Changeset, network::BazaClient, ping::Ping, Revision};

pub enum SyncAgent {
    InMemory { baza: Arc<Baza> },
    Network { client: BazaClient },
}

impl SyncAgent {
    pub fn new_in_memory(baza: Arc<Baza>) -> Result<Self> {
        Ok(SyncAgent::InMemory { baza })
    }

    pub fn new_in_network(client: BazaClient) -> Self {
        SyncAgent::Network { client }
    }

    pub async fn exchange_pings(&self, ping: &Ping) -> Result<Ping> {
        match self {
            SyncAgent::InMemory { baza, .. } => {
                let other_ping = ping;
                let ping = baza.get_connection()?.get_ping()?;

                if other_ping.rev.is_concurrent_or_newer_than(&ping.rev) {
                    baza.publish_event(BazaEvent::InstanceOutdated {})?;
                }

                Ok(ping)
            }
            SyncAgent::Network { client, .. } => client.exchange_pings(ping).await,
        }
    }

    pub async fn fetch_changes(&self, min_rev: &Revision) -> Result<Changeset> {
        match self {
            SyncAgent::InMemory { baza, .. } => baza.get_connection()?.get_changeset(min_rev),
            SyncAgent::Network { client, .. } => client.get_changeset(min_rev).await,
        }
    }

    pub async fn fetch_blob(&self, blob: &BLOB) -> Result<()> {
        match self {
            SyncAgent::InMemory { baza, .. } => {
                let other_blob = baza
                    .get_connection()?
                    .get_existing_blob(&blob.id)?
                    .context("requested BLOB must exist")?;

                tokio::fs::copy(&other_blob.file_path, &blob.file_path).await?;
            }
            SyncAgent::Network { client, .. } => {
                client.download_blob(blob).await?;
            }
        }

        Ok(())
    }
}

impl Display for SyncAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncAgent::InMemory { baza } => write!(
                f,
                "InMemoryAgent<{} {}>",
                baza.get_name(),
                baza.get_connection()
                    .and_then(|conn| conn.get_instance_id())
                    .expect("must read baza instance id")
            ),
            SyncAgent::Network { client } => write!(f, "NetworkAgent<{}>", client.get_url()),
        }
    }
}
