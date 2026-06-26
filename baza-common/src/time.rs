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
        let format = format_description::parse_borrowed::<3>(fmt)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(iso_str: &str) -> Timestamp {
        Timestamp::parse_iso8601_time(iso_str).unwrap()
    }

    #[test]
    fn format_time_uses_time_format_description() {
        let ts = timestamp("2023-10-23T11:23:39Z");

        assert_eq!(
            ts.format_time(
                "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
            )
            .unwrap(),
            "2023-10-23 11:23:39 +00"
        );
    }

    #[test]
    fn format_time_preserves_timestamp_offset() {
        let ts = timestamp("2023-10-23T11:23:39+03:30");

        assert_eq!(
            ts.format_time("[hour]:[minute] [offset_hour sign:mandatory]:[offset_minute]")
                .unwrap(),
            "11:23 +03:30"
        );
    }

    #[test]
    fn format_time_reports_invalid_format_descriptions() {
        let err = timestamp("2023-10-23T11:23:39Z")
            .format_time("[unknown component]")
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("Failed to parse format description [unknown component]"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn default_date_time_format_matches_documented_layout() {
        let ts = timestamp("2023-10-23T11:23:39Z");

        assert_eq!(ts.default_date_time_format(), "Mon Oct 23 11:23:39 2023");
    }

    #[test]
    fn default_date_time_format_space_pads_single_digit_days() {
        let ts = timestamp("2023-02-05T01:02:03Z");

        assert_eq!(ts.default_date_time_format(), "Sun Feb  5 01:02:03 2023");
    }

    #[test]
    fn parse_iso8601_time_accepts_rfc3339_offsets() {
        let ts = Timestamp::parse_iso8601_time("2023-10-23T11:23:39+03:30").unwrap();

        assert_eq!(
            ts.format_time("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]")
                .unwrap(),
            "2023-10-23T11:23:39+03:30"
        );
    }

    #[test]
    fn parse_iso8601_time_rejects_non_rfc3339_values() {
        let err = Timestamp::parse_iso8601_time("Mon Oct 23 11:23:39 2023").unwrap_err();

        assert!(
            err.to_string()
                .contains("Failed to parse time string as ISO8601"),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn timestamp_arithmetic_round_trips_std_duration() {
        let start = timestamp("2023-10-23T11:23:39Z");
        let end = start + Duration::from_secs(90);

        assert_eq!(end - start, time::Duration::seconds(90));
        assert_eq!(
            end.format_time("[hour]:[minute]:[second]").unwrap(),
            "11:25:09"
        );
    }

    #[test]
    fn display_uses_offset_date_time_representation() {
        assert_eq!(
            timestamp("2023-10-23T11:23:39+03:30").to_string(),
            "2023-10-23 11:23:39.0 +03:30:00"
        );
    }
}
