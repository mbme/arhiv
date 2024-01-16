use std::{sync::Arc, time::Duration};

use anyhow::{ensure, Result};
use tokio::{task::JoinHandle, time::interval};

use rs_utils::{log, now, FakeTime, Timestamp};

use crate::Baza;

pub type AutoCommitTask = JoinHandle<()>;

pub struct AutoCommitService {
    baza: Arc<Baza>,
    auto_commit_timeout: Duration,
    fake_time: Option<FakeTime>,
}

impl AutoCommitService {
    pub fn new(baza: Arc<Baza>, auto_commit_timeout: Duration) -> Self {
        AutoCommitService {
            baza,
            auto_commit_timeout,
            fake_time: None,
        }
    }

    pub fn with_fake_time(mut self) -> Self {
        self.fake_time = Some(FakeTime::new());

        self
    }

    fn get_time(&self) -> Timestamp {
        if let Some(ref fake_time) = self.fake_time {
            fake_time.now()
        } else {
            now()
        }
    }

    pub fn start(self) -> Result<AutoCommitTask> {
        let auto_commit_timeout = self.auto_commit_timeout;

        let task = tokio::spawn(async move {
            let mut interval = interval(self.auto_commit_timeout / 2);

            loop {
                interval.tick().await;

                if let Err(err) = self.try_auto_commit() {
                    log::warn!("Auto-commit failed: {err}");
                    break;
                }
            }

            log::debug!("Auto-commit task ended");
        });

        log::info!(
            "Started auto-commit service, auto-commit delay is {} seconds",
            auto_commit_timeout.as_secs()
        );

        Ok(task)
    }

    fn try_auto_commit(&self) -> Result<()> {
        let mut tx = self.baza.get_tx()?;

        let last_update_time = tx.get_last_update_time()?;
        let is_modified = tx.has_staged_documents()?;

        let time_since_last_update = (self.get_time() - last_update_time).to_std()?;

        let has_locks = !tx.list_document_locks()?.is_empty();

        if is_modified && !has_locks && time_since_last_update > self.auto_commit_timeout {
            log::debug!(
                "Starting auto-commit: {} seconds elapsed since last update",
                time_since_last_update.as_secs()
            );

            let documents_count = tx.commit_staged_documents()?;
            ensure!(
                documents_count > 0,
                "Expected a non-zero number of auto-committed documents"
            );
            log::info!("Auto-committed {documents_count} documents");

            tx.commit()?;
        } else {
            log::debug!("Nothing to auto-commit");
            tx.rollback()?;
        }

        Ok(())
    }
}
