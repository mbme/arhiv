use std::{sync::Mutex, time::Duration};

use chrono::{DateTime, Local, Utc};
use futures::Future;
use tokio::{
    task::JoinHandle,
    time::{sleep, Instant},
};

pub const MIN_TIMESTAMP: Timestamp = DateTime::<Utc>::MIN_UTC;

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}

pub fn format_time(timestamp: Timestamp, fmt: &str) -> String {
    timestamp.with_timezone(&Local).format(fmt).to_string()
}

// Mon Oct 23 11:23:39 2023 local time
pub fn default_date_time_format(timestamp: Timestamp) -> String {
    format_time(timestamp, "%a %b %e %T %Y")
}

// For use in tests, compatible with tokio::time::advance()
#[derive(Clone, Debug)]
pub struct FakeTime {
    start_time: Timestamp,
    tokio_instant: Instant,
}

impl FakeTime {
    pub fn new() -> Self {
        FakeTime {
            start_time: now(),
            tokio_instant: Instant::now(),
        }
    }

    pub fn now(&self) -> Timestamp {
        self.start_time + self.tokio_instant.elapsed()
    }
}

impl Default for FakeTime {
    fn default() -> Self {
        FakeTime::new()
    }
}

pub struct ScheduledTask {
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl ScheduledTask {
    pub fn new() -> Self {
        ScheduledTask {
            handle: Default::default(),
        }
    }

    pub fn schedule<F: Future<Output = ()> + Send + 'static>(
        &self,
        future_timeout: Duration,
        future: F,
    ) {
        self.cancel();

        let handle = tokio::spawn(async move {
            sleep(future_timeout).await;
            future.await;
        });

        self.handle.lock().expect("must lock").replace(handle);
    }

    pub fn cancel(&self) {
        if let Some(handle) = self.handle.lock().expect("must lock").take() {
            handle.abort();
        }
    }
}

impl Default for ScheduledTask {
    fn default() -> Self {
        ScheduledTask::new()
    }
}

impl Drop for ScheduledTask {
    fn drop(&mut self) {
        self.cancel();
    }
}
