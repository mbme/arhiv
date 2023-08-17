use std::cell::RefCell;

use anyhow::{bail, ensure, Context, Result};
use async_trait::async_trait;

use crate::entities::BLOB;

use super::{
    agent::SyncAgent, changeset::Changeset, instance_id::InstanceId, ping::Ping, Revision,
};

#[async_trait(?Send)]
pub trait SyncNetwork {
    async fn ping_all(&self, brief: &Ping) -> Result<()>;

    async fn pull_changes(&self, from: &InstanceId, min_rev: &Revision) -> Result<Changeset>;

    async fn fetch_blob(&self, from: &InstanceId, blob: &BLOB) -> Result<()>;
}

#[derive(Default)]
pub struct InMemorySyncNetwork<'n> {
    pub agents: RefCell<Vec<SyncAgent<'n, Self>>>,
}

impl<'n> InMemorySyncNetwork<'n> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_agent(&self, agent: SyncAgent<'n, Self>) {
        self.agents.borrow_mut().push(agent);
    }
}

#[async_trait(?Send)]
impl<'n> SyncNetwork for InMemorySyncNetwork<'n> {
    async fn ping_all(&self, brief: &Ping) -> Result<()> {
        let agents = self.agents.borrow();
        let local_agent = agents
            .iter()
            .find(|agent| agent.get_id() == &brief.instance_id)
            .context("can't find local agent")?;

        for agent in agents.iter() {
            if agent == local_agent {
                continue;
            }

            if let Some(answer_ping) = agent.handle_ping(brief.clone())? {
                let result = local_agent.handle_ping(answer_ping)?;

                ensure!(
                    result.is_none(),
                    "answer ping must be greater than local ping"
                );
            }
        }

        Ok(())
    }

    async fn pull_changes(&self, from: &InstanceId, min_rev: &Revision) -> Result<Changeset> {
        let agents = self.agents.borrow();
        let agent = agents
            .iter()
            .find(|agent| agent.get_id() == from)
            .context("failed to find matching instance")?;

        agent.handle_changes_request(min_rev)
    }

    async fn fetch_blob(&self, from: &InstanceId, blob: &BLOB) -> Result<()> {
        let agents = self.agents.borrow();
        let agent = agents
            .iter()
            .find(|agent| agent.get_id() == from)
            .context("failed to find matching instance")?;

        if let Some(other_blob) = agent.handle_blob_request(&blob.id)? {
            tokio::fs::copy(&other_blob.file_path, &blob.file_path).await?;
        } else {
            bail!("Instance {} doesn't have BLOB {}", from, blob.id);
        }

        Ok(())
    }
}
