use rs_utils::Timestamp;

use crate::sync::InstanceId;

use super::kvs::KvsConstKey;

pub const SETTINGS_NAMESPACE: &str = "settings";

pub const SETTING_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "data_version");

pub const SETTING_COMPUTED_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "computed_data_version");

pub const SETTING_INSTANCE_ID: &KvsConstKey<InstanceId> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "instance_id");

pub const SETTING_LAST_SYNC_TIME: &KvsConstKey<Timestamp> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "last_sync_time");
