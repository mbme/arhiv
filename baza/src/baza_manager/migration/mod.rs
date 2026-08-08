mod asset_content_sha256;

use anyhow::{Result, bail};
use baza_storage::crypto::age::AgeKey;

use super::BazaManager;

impl BazaManager {
    /// Applies required data migrations before normal state loading enforces the latest schema.
    pub(super) fn migrate_to_latest_data_version_if_needed(&self, key: &AgeKey) -> Result<bool> {
        match self.schema.get_latest_data_version() {
            2 => self.migrate_data_v1_to_v2_asset_content_sha256_with_key(key),
            latest => bail!("No data migration path is registered for data version {latest}"),
        }
    }
}
