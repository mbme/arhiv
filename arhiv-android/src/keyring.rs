use std::sync::{Arc, RwLock};

use anyhow::{Context, Result, anyhow, ensure};
use jni::{
    JavaVM, jni_sig, jni_str,
    objects::{Global, JObject, JValue},
};

use arhiv::{ArhivKeyring, Keyring};
use baza_common::{ExposeSecret, SecretString, log};

/// This implementation of Keyring only receives password once on init, from Android.
/// The reason is that the biometric auth process in Android is asynchronous, so the easiest approach
/// is to do it only once on app init, and then just update the local password copy.
/// Similarly, the set_string() also starts an async process to encrypt & save the password,
/// but doesn't wait for results. So the password may not actually be saved, even if the method call didn't fail.
pub struct AndroidKeyring {
    password: RwLock<Option<SecretString>>,
    android_controller: Global<JObject<'static>>, // instance of AndroidController
    jvm: JavaVM,
}

impl AndroidKeyring {
    pub fn new_arhiv_keyring(
        password: Option<SecretString>,
        android_controller: Global<JObject<'static>>,
        jvm: JavaVM,
    ) -> ArhivKeyring {
        let keyring = AndroidKeyring {
            password: RwLock::new(password),
            android_controller,
            jvm,
        };

        ArhivKeyring::new(Arc::new(keyring))
    }
}

impl Keyring for AndroidKeyring {
    /// NOTE: Only password is supported
    fn get_string(&self, name: &str) -> Result<Option<SecretString>> {
        match name {
            ArhivKeyring::PASSWORD => {
                let password_guard = self.password.read().map_err(|err| {
                    anyhow!("Failed to acquire read lock for the password: {err}")
                })?;

                Ok(password_guard.clone())
            }
            _ => {
                unreachable!("Got unexpected entry {name}");
            }
        }
    }

    /// NOTE: Only password is supported
    fn set_string(&self, name: &str, value: Option<SecretString>) -> Result<()> {
        log::info!("Saving {name} to Android keyring");

        ensure!(
            name == ArhivKeyring::PASSWORD,
            "Can change only password entry, got {name}"
        );

        let mut password_guard = self
            .password
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the password: {err}"))?;

        self.jvm
            .attach_current_thread(|env| -> Result<()> {
                let null_password = JObject::null();
                let password_jstring = value.as_ref().map(|p| {
                    env.new_string(p.expose_secret())
                        .expect("Couldn't create java String")
                });
                let password_arg = match password_jstring.as_ref() {
                    Some(password_jstring) => JValue::from(password_jstring),
                    None => JValue::from(&null_password),
                };

                env.call_method(
                    &self.android_controller,
                    jni_str!("savePassword"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[password_arg],
                )
                .context("Failed to call AndroidController.savePassword()")?;

                Ok(())
            })
            .context("Failed to attach current thread to JavaVM")?;

        *password_guard = value;

        Ok(())
    }
}
