use crate::sync::InstanceId;

use super::kvs::KvsConstKey;

pub const SETTINGS_NAMESPACE: &'static str = "settings";

pub const SETTING_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "data_version");

pub const SETTING_COMPUTED_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "computed_data_version");

pub const SETTING_INSTANCE_ID: &KvsConstKey<InstanceId> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "instance_id");
