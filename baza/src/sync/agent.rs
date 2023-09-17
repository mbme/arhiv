use std::{rc::Rc, str::FromStr};

use anyhow::{Context, Result};
use reqwest::Url;

use crate::{entities::BLOB, Baza};

use super::{changeset::Changeset, network::BazaClient, ping::Ping, Revision};

pub enum SyncAgent {
    InMemory { baza: Rc<Baza> },
    Network { client: BazaClient },
}

impl SyncAgent {
    pub fn parse_network_agents(urls: &[&str], downloads_dir: &str) -> Result<Vec<Self>> {
        urls.iter()
            .map(|url| {
                let client = BazaClient::new(
                    Url::from_str(url).context("failed to parse url")?,
                    downloads_dir,
                );

                Ok(Self::new_in_network(client))
            })
            .collect()
    }

    pub fn new_in_memory(baza: Rc<Baza>) -> Result<Self> {
        Ok(SyncAgent::InMemory { baza })
    }

    pub fn new_in_network(client: BazaClient) -> Self {
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
