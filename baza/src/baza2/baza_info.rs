use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BazaInfo {
    pub storage_version: u8,
    pub data_version: u8,
}

impl BazaInfo {
    #[cfg(test)]
    pub fn new_test_info() -> Self {
        Self {
            data_version: 1,
            storage_version: 1,
        }
    }
}
