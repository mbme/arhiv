use std::{fmt::Display, str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use reqwest::Url;

use crate::{
    entities::{InstanceId, Revision, BLOB},
    Baza, BazaEvent,
};

use super::{changeset::Changeset, network::BazaClient, ping::Ping};

#[derive(Clone)]
pub enum SyncAgent {
    InMemory {
        baza: Arc<Baza>,
        instance_id: InstanceId,
    },
    Network {
        client: BazaClient,
        instance_id: InstanceId,
    },
}

impl SyncAgent {
    pub fn new_in_memory(baza: Arc<Baza>) -> Result<Self> {
        let instance_id = baza.get_connection()?.get_instance_id()?;

        Ok(SyncAgent::InMemory { baza, instance_id })
    }

    pub fn new_in_network(instance_id: InstanceId, url: &str, downloads_dir: &str) -> Result<Self> {
        let client = BazaClient::new(
            Url::from_str(url).context("failed to parse url")?,
            downloads_dir,
        );

        Ok(SyncAgent::Network {
            client,
            instance_id,
        })
    }

    pub fn get_instance_id(&self) -> &InstanceId {
        match self {
            SyncAgent::InMemory { instance_id, .. } => instance_id,
            SyncAgent::Network { instance_id, .. } => instance_id,
        }
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
            SyncAgent::InMemory { baza, instance_id } => {
                write!(f, "InMemoryAgent<{} {}>", baza.get_app_name(), instance_id)
            }
            SyncAgent::Network {
                client,
                instance_id,
            } => write!(f, "NetworkAgent<{} {}>", client.get_url(), instance_id),
        }
    }
}
