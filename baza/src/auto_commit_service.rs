use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tokio::{task::JoinHandle, time::interval};

use rs_utils::{log, now, FakeTime, Timestamp, MIN_TIMESTAMP};

use crate::baza2::BazaManager;

pub type AutoCommitTask = JoinHandle<()>;

pub struct AutoCommitService {
    baza_manager: Arc<BazaManager>,
    auto_commit_timeout: Duration,
    fake_time: Option<FakeTime>,
    last_known_modification_time: Timestamp,
}

impl AutoCommitService {
    pub const DEFAULT_AUTO_COMMIT_DELAY: Duration = Duration::from_secs(600);

    pub fn new(baza_manager: Arc<BazaManager>, auto_commit_timeout: Duration) -> Self {
        AutoCommitService {
            baza_manager,
            auto_commit_timeout,
            fake_time: None,
            last_known_modification_time: MIN_TIMESTAMP,
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

    pub fn start(mut self) -> Result<AutoCommitTask> {
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

    fn try_auto_commit(&mut self) -> Result<()> {
        if !self.baza_manager.storage_exists()? {
            log::trace!("Auto-commit: storage doesn't exist");
            return Ok(());
        }

        let state_modification_time = self.baza_manager.get_state_file_modification_time()?;

        if self.last_known_modification_time == state_modification_time {
            log::trace!("Auto-commit: state file didn't change since last check");
            return Ok(());
        }

        self.last_known_modification_time = state_modification_time;

        let time_since_last_update = (self.get_time() - state_modification_time).to_std()?;

        if time_since_last_update > self.auto_commit_timeout {
            log::debug!(
                "Auto-commit: {} seconds elapsed since last update",
                time_since_last_update.as_secs()
            );

            let mut baza = self.baza_manager.open()?;
            baza.commit()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use anyhow::Result;
    use rs_utils::TempFile;
    use tokio::time::{advance, sleep};

    use crate::{baza2::BazaManager, tests::new_empty_document, AutoCommitService};

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn test_auto_commit_on_start() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = Arc::new(BazaManager::new_for_tests(&temp_dir.path));
        let mut baza = manager.open().unwrap();

        baza.stage_document(new_empty_document(), &None)?;
        baza.save_changes()?;
        drop(baza);

        let auto_commit_timeout = Duration::from_secs(10);
        let service = AutoCommitService::new(manager.clone(), auto_commit_timeout).with_fake_time();

        {
            let baza = manager.open().unwrap();
            assert!(baza.has_staged_documents());
        }

        advance(auto_commit_timeout * 2).await;

        service.start()?;

        sleep(Duration::from_secs(1)).await;

        {
            let baza = manager.open().unwrap();
            assert!(!baza.has_staged_documents());
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn test_schedule_auto_commit() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = Arc::new(BazaManager::new_for_tests(&temp_dir.path));

        let auto_commit_timeout = Duration::from_secs(10);
        let service = AutoCommitService::new(manager.clone(), auto_commit_timeout).with_fake_time();

        {
            let mut baza = manager.open().unwrap();
            baza.stage_document(new_empty_document(), &None)?;
            assert!(baza.has_staged_documents());
        }

        advance(auto_commit_timeout * 2).await;

        service.start()?;

        sleep(Duration::from_secs(1)).await;

        {
            let baza = manager.open().unwrap();
            assert!(!baza.has_staged_documents());
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn test_reschedule_auto_commit_on_state_modification() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = Arc::new(BazaManager::new_for_tests(&temp_dir.path));

        let auto_commit_timeout = Duration::from_secs(10);
        let service = AutoCommitService::new(manager.clone(), auto_commit_timeout).with_fake_time();

        service.start()?;

        sleep(Duration::from_secs(1)).await;

        {
            let mut baza = manager.open().unwrap();
            baza.stage_document(new_empty_document(), &None)?;
            baza.save_changes().unwrap();
        }

        advance(auto_commit_timeout - Duration::from_secs(2)).await;
        sleep(Duration::from_secs(1)).await;

        {
            let mut baza = manager.open().unwrap();

            assert!(baza.has_staged_documents());

            baza.stage_document(new_empty_document(), &None)?;
            baza.save_changes().unwrap();
        }

        advance(auto_commit_timeout - Duration::from_secs(2)).await;
        sleep(Duration::from_secs(1)).await;

        {
            let baza = manager.open().unwrap();
            assert!(baza.has_staged_documents());
        }

        Ok(())
    }
}
