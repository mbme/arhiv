use std::{sync::Arc, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

use rs_utils::{log, now, FakeTime, Timestamp};

use crate::{baza::BazaEvent, Baza};

#[derive(Clone)]
pub struct AutoCommitService {
    baza: Arc<Baza>,
    auto_commit_timeout: Duration,
    cancellation_token: CancellationToken,
    fake_time: Option<FakeTime>,
    started: bool,
}

impl AutoCommitService {
    pub fn new(baza: Arc<Baza>, auto_commit_timeout: Duration) -> Self {
        AutoCommitService {
            baza,
            auto_commit_timeout,
            cancellation_token: CancellationToken::new(),
            fake_time: None,
            started: false,
        }
    }

    pub fn with_fake_time(mut self) -> Self {
        self.fake_time = Some(FakeTime::new());

        self
    }

    pub fn get_auto_commit_timeout(&self) -> Duration {
        self.auto_commit_timeout
    }

    fn get_time(&self) -> Timestamp {
        if let Some(ref fake_time) = self.fake_time {
            fake_time.now()
        } else {
            now()
        }
    }

    fn schedule_task(&self, duration: Duration) -> JoinHandle<()> {
        let service = self.clone();
        tokio::spawn(async move {
            log::debug!(
                "Scheduled auto-commit attempt in {} seconds",
                duration.as_secs()
            );

            sleep(duration).await;

            if let Err(err) = service.try_auto_commit() {
                log::warn!("Auto-commit failed: {err}");
            }
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if self.started {
            bail!("Already started");
        }

        self.started = true;

        self.try_auto_commit().context("Auto-commit failed")?;

        let service = self.clone();

        tokio::spawn(async move { service.start_watch_task().await });

        log::info!("Started auto-commit service");

        Ok(())
    }

    async fn start_watch_task(&self) -> Result<()> {
        let mut task: Option<JoinHandle<()>> = {
            let conn = self.baza.get_connection()?;

            let last_update_time = conn.get_last_update_time()?;
            let is_modified = conn.has_staged_documents()?;

            let time_since_last_update = (self.get_time() - last_update_time).to_std()?;

            if is_modified {
                ensure!(time_since_last_update < self.auto_commit_timeout);

                let auto_commit_timeout =
                    (self.auto_commit_timeout - time_since_last_update) + Duration::from_secs(1);

                Some(self.schedule_task(auto_commit_timeout))
            } else {
                None
            }
        };

        let mut events = self.baza.get_events_channel();

        loop {
            tokio::select! {
                event = events.recv() => {
                    log::debug!("Watch task got Baza event {event:#?}");

                    match event {
                        Ok(BazaEvent::DocumentStaged {}) => {
                            if let Some(ref task) = task {
                                log::debug!("Aborting pending auto-commit task");
                                task.abort();
                            }

                            task = Some(self.schedule_task(self.auto_commit_timeout));
                        },
                        Ok(_) => {},
                        Err(_) => {
                            break;
                        },
                    }
                },

                _ = self.cancellation_token.cancelled() => {
                    log::debug!("Watch task got cancelled");
                    break;
                }
            }
        }

        if let Some(ref task) = task {
            log::debug!("Aborting pending auto-commit task");
            task.abort();
        }

        log::debug!("Watch task ended");

        Ok(())
    }

    pub fn stop(self) {
        self.cancellation_token.cancel();
    }

    fn try_auto_commit(&self) -> Result<()> {
        let mut tx = self.baza.get_tx()?;

        let last_update_time = tx.get_last_update_time()?;
        let is_modified = tx.has_staged_documents()?;

        if is_modified && last_update_time + self.auto_commit_timeout < self.get_time() {
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

impl Drop for AutoCommitService {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
