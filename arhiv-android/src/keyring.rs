use std::sync::{Arc, RwLock};

use anyhow::{Context, Result, anyhow, ensure};
use jni::{
    JavaVM, jni_sig, jni_str,
    objects::{Global, JObject, JValue},
};

use arhiv::{ArhivKeyring, Keyring};
use baza_common::{ExposeSecret, SecretString, log};

/// This implementation of Keyring only receives a storage key once on init, from Android.
/// The reason is that the biometric auth process in Android is asynchronous, so the easiest approach
/// is to do it only once on app init, and then just update the local storage-key copy.
/// Saving starts an asynchronous biometric-gated operation and is not acknowledged. Deletion is
/// synchronous and fallible so logout is not reported until the stored credential is removed.
pub struct AndroidKeyring {
    storage_key: RwLock<Option<SecretString>>,
    android_controller: Global<JObject<'static>>, // instance of AndroidController
    jvm: JavaVM,
}

impl AndroidKeyring {
    pub fn new_arhiv_keyring(
        storage_key: Option<SecretString>,
        android_controller: Global<JObject<'static>>,
        jvm: JavaVM,
    ) -> ArhivKeyring {
        let keyring = AndroidKeyring {
            storage_key: RwLock::new(storage_key),
            android_controller,
            jvm,
        };

        ArhivKeyring::new(Arc::new(keyring))
    }
}

impl Keyring for AndroidKeyring {
    fn get_string(&self, name: &str) -> Result<Option<SecretString>> {
        match name {
            ArhivKeyring::STORAGE_KEY => {
                let storage_key_guard = self.storage_key.read().map_err(|err| {
                    anyhow!("Failed to acquire read lock for the storage key: {err}")
                })?;

                Ok(storage_key_guard.clone())
            }
            _ => {
                unreachable!("Got unexpected entry {name}");
            }
        }
    }

    fn set_string(&self, name: &str, value: Option<SecretString>) -> Result<()> {
        log::info!("Saving {name} to Android keyring");

        ensure!(
            name == ArhivKeyring::STORAGE_KEY,
            "Can change only storage key entry, got {name}"
        );

        let mut storage_key_guard = self
            .storage_key
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the storage key: {err}"))?;

        self.jvm
            .attach_current_thread(|env| -> Result<()> {
                let null_storage_key = JObject::null();
                let storage_key_jstring = value.as_ref().map(|storage_key| {
                    env.new_string(storage_key.expose_secret())
                        .expect("Couldn't create java String")
                });
                let storage_key_arg = match storage_key_jstring.as_ref() {
                    Some(storage_key_jstring) => JValue::from(storage_key_jstring),
                    None => JValue::from(&null_storage_key),
                };

                env.call_method(
                    &self.android_controller,
                    jni_str!("saveStorageKey"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[storage_key_arg],
                )
                .context("Failed to call AndroidController.saveStorageKey()")?;

                Ok(())
            })
            .context("Failed to attach current thread to JavaVM")?;

        *storage_key_guard = value;

        Ok(())
    }
}
