use chrono::{DateTime, Utc};

pub const MIN_TIMESTAMP: Timestamp = DateTime::<Utc>::MIN_UTC;

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}
