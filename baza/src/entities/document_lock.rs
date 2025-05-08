use std::fmt::Display;

use serde::{Deserialize, Serialize};

use rs_utils::{Timestamp, generate_random_id};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentLockKey(String);

impl DocumentLockKey {
    pub const LENGTH: usize = 14;

    pub fn new_random_key() -> Self {
        Self(generate_random_id(DocumentLockKey::LENGTH))
    }

    pub fn from_string(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Display for DocumentLockKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DocumentLock {
    key: DocumentLockKey,
    lock_time: Timestamp,
    reason: String,
}

impl DocumentLock {
    #[must_use]
    pub fn new(reason: String) -> Self {
        DocumentLock {
            key: DocumentLockKey::new_random_key(),
            lock_time: Timestamp::now(),
            reason,
        }
    }

    #[must_use]
    pub fn get_key(&self) -> &DocumentLockKey {
        &self.key
    }

    #[must_use]
    pub fn is_valid_key(&self, key: &DocumentLockKey) -> bool {
        self.key == *key
    }
}

impl Display for DocumentLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}]: {}",
            self.lock_time.default_date_time_format(),
            self.key,
            self.reason
        )
    }
}
