use std::{
    fmt::Display,
    ops::{Add, Sub},
    sync::Mutex,
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result, anyhow};
use futures::Future;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description};
use tokio::{
    task::JoinHandle,
    time::{Instant, sleep},
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[serde(transparent)]
pub struct Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

impl Timestamp {
    pub const MIN: Timestamp = Timestamp(OffsetDateTime::UNIX_EPOCH);

    pub fn now() -> Timestamp {
        Timestamp(OffsetDateTime::now_local().expect("Must create local timestamp"))
    }

    pub fn parse_iso8601_time(iso_str: &str) -> Result<Timestamp> {
        let ts = OffsetDateTime::parse(iso_str, &format_description::well_known::Rfc3339)
            .context("Failed to parse time string as ISO8601")?;

        Ok(Timestamp(ts))
    }

    pub fn format_time(&self, fmt: &str) -> Result<String> {
        let format = format_description::parse(fmt)
            .context(anyhow!("Failed to parse format description {fmt}"))?;

        self.0
            .format(&format)
            .context(anyhow!("Failed to format timestamp with {fmt}"))
    }

    // Mon Oct 23 11:23:39 2023 local time
    pub fn default_date_time_format(&self) -> String {
        self.format_time("[weekday repr:short] [month repr:short] [day padding:space] [hour]:[minute]:[second] [year]").expect("default date time format must be valid")
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp(self.0 + rhs)
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = time::Duration;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        self.0 - rhs.0
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        Timestamp(value.into())
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
            start_time: Timestamp::now(),
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
