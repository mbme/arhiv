use std::marker::PhantomData;

use baza::DBSetting;
use rs_utils::Timestamp;

pub const SETTING_IS_PRIME: DBSetting<bool> = DBSetting("is_prime", PhantomData);

pub const SETTING_LAST_SYNC_TIME: DBSetting<Timestamp> = DBSetting("last_sync_time", PhantomData);
