use anyhow::Result;

use crate::entities::InstanceId;

use super::kvs::KvsConstKey;
use super::BazaConnection;

pub const SETTINGS_NAMESPACE: &str = "settings";

const SETTING_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "data_version");

const SETTING_COMPUTED_DATA_VERSION: &KvsConstKey<u8> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "computed_data_version");

const SETTING_INSTANCE_ID: &KvsConstKey<InstanceId> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "instance_id");

impl BazaConnection {
    pub fn get_instance_id(&self) -> Result<InstanceId> {
        self.kvs_const_must_get(SETTING_INSTANCE_ID)
    }

    pub(crate) fn set_instance_id(&self, instance_id: &InstanceId) -> Result<()> {
        self.kvs_const_set(SETTING_INSTANCE_ID, instance_id)
    }

    pub fn get_data_version(&self) -> Result<u8> {
        self.kvs_const_must_get(SETTING_DATA_VERSION)
    }

    pub(crate) fn set_data_version(&self, version: u8) -> Result<()> {
        self.kvs_const_set(SETTING_DATA_VERSION, &version)
    }

    pub fn get_computed_data_version(&self) -> Result<u8> {
        let computed_data_version = self
            .kvs_const_get(SETTING_COMPUTED_DATA_VERSION)?
            .unwrap_or(0);

        Ok(computed_data_version)
    }

    pub(crate) fn set_computed_data_version(&self, version: u8) -> Result<()> {
        self.kvs_const_set(SETTING_COMPUTED_DATA_VERSION, &version)
    }
}
