use std::rc::Rc;

use anyhow::{Context, Result};

use crate::{entities::BLOB, Baza};

use super::{changeset::Changeset, network::BazaRpcClient, ping::Ping, Revision};

pub enum SyncAgent {
    InMemory { baza: Rc<Baza> },
    Network { client: BazaRpcClient },
}

impl SyncAgent {
    pub fn new_in_memory(baza: Rc<Baza>) -> Result<Self> {
        Ok(SyncAgent::InMemory { baza })
    }

    pub fn new_in_network(client: BazaRpcClient) -> Self {
        SyncAgent::Network { client }
    }

    pub async fn fetch_ping(&self) -> Result<Ping> {
        match self {
            SyncAgent::InMemory { baza, .. } => baza.get_connection()?.get_ping(),
            SyncAgent::Network { client, .. } => client.get_ping().await,
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
