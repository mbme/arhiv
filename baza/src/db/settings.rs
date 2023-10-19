use anyhow::Result;

use rs_utils::Timestamp;

use crate::entities::InstanceId;

use super::kvs::{KvsConstKey, KvsEntry};
use super::BazaConnection;

const SETTINGS_NAMESPACE: &str = "settings";

const SETTING_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "data_version");

const SETTING_COMPUTED_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "computed_data_version");

const SETTING_INSTANCE_ID: &KvsConstKey<InstanceId> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "instance_id");

const SETTING_LAST_SYNC_TIME: &KvsConstKey<Timestamp> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "last_sync_time");

impl BazaConnection {
    pub fn list_settings(&self) -> Result<Vec<KvsEntry>> {
        self.kvs_list(Some(SETTINGS_NAMESPACE))
    }

    pub fn get_instance_id(&self) -> Result<InstanceId> {
        self.kvs_const_must_get(SETTING_INSTANCE_ID)
    }

    pub fn get_data_version(&self) -> Result<u8> {
        self.kvs_const_must_get(SETTING_DATA_VERSION)
    }

    pub(crate) fn set_data_version(&self, version: u8) -> Result<()> {
        self.kvs_const_set(SETTING_DATA_VERSION, &version)
    }

    pub(crate) fn get_computed_data_version(&self) -> Result<u8> {
        let computed_data_version = self
            .kvs_const_get(SETTING_COMPUTED_DATA_VERSION)?
            .unwrap_or(0);

        Ok(computed_data_version)
    }

    pub(crate) fn set_computed_data_version(&self, version: u8) -> Result<()> {
        self.kvs_const_set(SETTING_COMPUTED_DATA_VERSION, &version)
    }

    pub(crate) fn set_instance_id(&self, instance_id: &InstanceId) -> Result<()> {
        self.kvs_const_set(SETTING_INSTANCE_ID, instance_id)
    }

    pub fn get_last_sync_time(&self) -> Result<Timestamp> {
        self.kvs_const_must_get(SETTING_LAST_SYNC_TIME)
    }

    pub(crate) fn set_last_sync_time(&self, last_sync_time: &Timestamp) -> Result<()> {
        self.kvs_const_set(SETTING_LAST_SYNC_TIME, last_sync_time)
    }
}
