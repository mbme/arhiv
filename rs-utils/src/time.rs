use chrono::{DateTime, Local, Utc};

pub const MIN_TIMESTAMP: Timestamp = DateTime::<Utc>::MIN_UTC;

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}

pub fn format_time(timestamp: Timestamp, fmt: &str) -> String {
    timestamp.with_timezone(&Local).format(fmt).to_string()
}
