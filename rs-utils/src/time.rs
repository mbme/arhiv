use chrono::{DateTime, Local, Utc};
use tokio::time::Instant;

pub const MIN_TIMESTAMP: Timestamp = DateTime::<Utc>::MIN_UTC;

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}

pub fn format_time(timestamp: Timestamp, fmt: &str) -> String {
    timestamp.with_timezone(&Local).format(fmt).to_string()
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
