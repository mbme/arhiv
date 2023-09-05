use baza::{KvsConstKey, SETTINGS_NAMESPACE};
use rs_utils::Timestamp;

pub const SETTING_LAST_SYNC_TIME: &KvsConstKey<Timestamp> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "last_sync_time");
