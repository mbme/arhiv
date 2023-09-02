use std::rc::Rc;

use anyhow::{Context, Result};

use crate::{entities::BLOB, Baza};

use super::{
    changeset::Changeset, instance_id::InstanceId, network::BazaRpcClient, ping::Ping, Revision,
};

pub enum SyncAgent {
    InMemory {
        id: InstanceId,
        baza: Rc<Baza>,
    },
    Network {
        id: InstanceId,
        client: BazaRpcClient,
    },
}

impl SyncAgent {
    pub fn new_in_memory(baza: Rc<Baza>) -> Result<Self> {
        let id = baza.get_connection()?.get_instance_id()?;

        Ok(SyncAgent::InMemory { id, baza })
    }

    pub fn new_in_network(id: InstanceId, client: BazaRpcClient) -> Self {
        SyncAgent::Network { id, client }
    }

    pub fn get_id(&self) -> &InstanceId {
        match self {
            SyncAgent::InMemory { id, .. } => id,
            SyncAgent::Network { id, .. } => id,
        }
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

impl PartialEq for SyncAgent {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}
