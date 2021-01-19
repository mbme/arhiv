use crate::entities::{Revision, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DbStatus {
    pub arhiv_id: String,
    pub is_prime: bool,
    pub schema_version: u8,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
}

impl DbStatus {
    pub fn get_prime_status(&self) -> &str {
        if self.is_prime {
            "prime"
        } else {
            "replica"
        }
    }
}
