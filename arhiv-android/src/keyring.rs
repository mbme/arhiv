use std::sync::{Arc, RwLock};

use anyhow::{anyhow, ensure, Context, Result};
use jni::{
    objects::{GlobalRef, JObject, JString},
    JavaVM,
};

use arhiv::ArhivKeyring;
use rs_utils::{keyring::Keyring, log, ExposeSecret, SecretString};

/// This implementation of Keyring only receives password once on init, from Android.
/// The reason is that the biometric auth process in Android is asynchronous, so the easiest approach
/// is to do it only once on app init, and then just update the local password copy.
/// Similarly, the set_string() also starts an async process to encrypt & save the password,
/// but doesn't wait for results. So the password may not actually be saved, even if the method call didn't fail.
pub struct AndroidKeyring {
    password: RwLock<Option<SecretString>>,
    android_controller: GlobalRef, // instance of AndroidController
    jvm: JavaVM,
}

impl AndroidKeyring {
    pub fn new_arhiv_keyring(
        password: Option<SecretString>,
        android_controller: GlobalRef,
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

        let _guard = self
            .jvm
            .attach_current_thread()
            .context("Failed to attach current thread to JavaVM");

        let mut env = self
            .jvm
            .get_env()
            .expect("Current thread must be attached to JavaVM to get JNIEnv");

        let password_jstring: JString = match value {
            Some(ref p) => env
                .new_string(p.expose_secret())
                .expect("Couldn't create java String"),
            None => JObject::null().into(),
        };

        env.call_method(
            &self.android_controller,
            "savePassword",
            "(Ljava/lang/String;)V",
            &[(&password_jstring).into()],
        )
        .context("Failed to call AndroidController.savePassword()")?;

        *password_guard = value;

        Ok(())
    }
}
