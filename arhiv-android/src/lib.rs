use std::{
    sync::{Arc, LazyLock, Mutex, RwLock},
    time::Duration,
};

use anyhow::{anyhow, ensure, Context, Result};
use jni::{
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    JNIEnv, JavaVM,
};
use tokio::runtime::Runtime;

use arhiv::{ArhivOptions, ArhivServer, Keyring, ServerInfo};
use rs_utils::{log, ExposeSecret, SecretString};

static RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| Mutex::new(None));
static ARHIV_SERVER: LazyLock<Mutex<Option<ArhivServer>>> = LazyLock::new(|| Mutex::new(None));

fn start_server(options: ArhivOptions) -> Result<ServerInfo> {
    let mut runtime_lock = RUNTIME
        .lock()
        .map_err(|err| anyhow!("Failed to lock RUNTIME: {err}"))?;
    ensure!(runtime_lock.is_none(), "Runtime already started");

    let mut server_lock = ARHIV_SERVER
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV_SERVER: {err}"))?;
    ensure!(server_lock.is_none(), "Server already started");

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    let runtime = builder.build().context("failed to create tokio runtime")?;

    let _guard = runtime.enter();

    let server = runtime.block_on(ArhivServer::start(options, 0))?;

    if cfg!(test) {
        server.arhiv.baza.create("test1234".into())?;
    }
    let server_info = server
        .arhiv
        .collect_server_info()?
        .context("Failed to collect server info")?;

    *server_lock = Some(server);
    *runtime_lock = Some(runtime);

    Ok(server_info)
}

fn stop_server() -> Result<()> {
    let mut server_lock = ARHIV_SERVER
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV_SERVER: {err}"))?;

    let server = server_lock.take().context("Server is missing")?;

    let mut runtime_lock = RUNTIME
        .lock()
        .map_err(|err| anyhow!("Failed to lock RUNTIME: {err}"))?;

    let runtime = runtime_lock.take().context("Runtime is missing")?;

    runtime.block_on(server.shutdown())?;
    runtime.shutdown_timeout(Duration::from_millis(500));

    Ok(())
}

/// This implementation of Keyring only receives password once on init, from Android.
/// The reason is that the biometric auth process in Android is asynchronous, so the easiest approach
/// is to do it only once on app init, and then just update the local password copy.
/// Similarly, the set_password() also starts an async process to encrypt & save the password,
/// but doesn't wait for results. So the password may not actually be saved, even if the method call didn't fail.
struct AndroidKeyring {
    password: RwLock<Option<SecretString>>,
    android_controller: GlobalRef, // instance of AndroidController
    jvm: JavaVM,
}

impl Keyring for AndroidKeyring {
    fn get_password(&self) -> Result<Option<SecretString>> {
        let password_guard = self
            .password
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the password: {err}"))?;

        Ok(password_guard.clone())
    }

    fn set_password(&self, password: Option<SecretString>) -> Result<()> {
        log::info!("Saving password to Android keyring");

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

        let password_jstring: JString = match password {
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

        *password_guard = password;

        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass,
    app_files_dir: JString,
    external_storage_dir: JString,
    password: JString,
    android_controller: JObject, // AndroidController
) -> JObject<'local> {
    log::setup_android_logger("me.mbsoftware.arhiv");
    log::setup_panic_hook();

    let app_files_dir: String = env
        .get_string(&app_files_dir)
        .expect("Must read JNI string app_files_dir")
        .into();
    log::debug!("Files dir: {app_files_dir}");

    let external_storage_dir: String = env
        .get_string(&external_storage_dir)
        .expect("Must read JNI string external_storage_dir")
        .into();
    log::debug!("Storage dir: {external_storage_dir}");

    let password: Option<SecretString> = if password.as_raw().is_null() {
        None
    } else {
        let password: String = env
            .get_string(&password)
            .expect("Must read JNI string password")
            .into();

        Some(password.into())
    };
    let android_controller = env
        .new_global_ref(android_controller)
        .expect("Must turn AndroidController instance into global ref");
    let jvm = env.get_java_vm().expect("Can't get reference to JVM");
    let keyring = AndroidKeyring {
        password: RwLock::new(password),
        android_controller,
        jvm,
    };
    let options = ArhivOptions {
        storage_dir: format!("{external_storage_dir}/Arhiv"),
        state_dir: app_files_dir,
        downloads_dir: format!("{external_storage_dir}/Downloads"), // TODO pass from Android
        file_browser_root_dir: external_storage_dir,
        keyring: Arc::new(keyring),
    };

    let server_info = start_server(options).expect("must start server");

    // Create an instance of me.mbsoftware.arhiv.ServerInfo using JNI
    let server_info_class = env
        .find_class("me/mbsoftware/arhiv/ServerInfo")
        .expect("Couldn't find ServerInfo class");
    let server_info_object = env
        .alloc_object(&server_info_class)
        .expect("Couldn't allocate ServerInfo object");

    // Set ServerInfo.uiUrl field on the Java object
    let ui_url_field = env
        .get_field_id(&server_info_class, "uiUrl", "Ljava/lang/String;")
        .expect("Couldn't find object field String uiUrl");
    let ui_url = env
        .new_string(server_info.ui_url)
        .expect("Couldn't create java String!");
    env.set_field_unchecked(&server_info_object, ui_url_field, JValue::from(&ui_url))
        .expect("Couldn't set field String uiUrl");

    // Set ServerInfo.authToken field on the Java object
    let auth_token_field = env
        .get_field_id(&server_info_class, "authToken", "Ljava/lang/String;")
        .expect("Couldn't find object field String authToken");
    let auth_token = env
        .new_string(server_info.auth_token)
        .expect("Couldn't create java String!");
    env.set_field_unchecked(
        &server_info_object,
        auth_token_field,
        JValue::from(&auth_token),
    )
    .expect("Couldn't set field String authToken");

    // Set ServerInfo.certificate field on the Java object
    let certificate_field = env
        .get_field_id(&server_info_class, "certificate", "[B") // byte[] in Java
        .expect("Couldn't find object field byte[] certificate");
    let certificate = env
        .byte_array_from_slice(&server_info.certificate)
        .expect("Couldn't create java byte[]!");
    env.set_field_unchecked(
        &server_info_object,
        certificate_field,
        JValue::from(&certificate),
    )
    .expect("Couldn't set field byte[] certificate");

    server_info_object
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_stopServer() {
    stop_server().expect("must stop server");
    log::info!("Stopped server");
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::{sync::Arc, thread};

    use arhiv::{ArhivOptions, NoopKeyring};
    use rs_utils::TempFile;

    use crate::{start_server, stop_server};

    #[test]
    fn test_arhiv_server_for_android() {
        let temp_dir = TempFile::new_with_details("AndroidTest", "");
        temp_dir.mkdir().expect("must create temp dir");

        let options = ArhivOptions {
            storage_dir: format!("{temp_dir}/storage"),
            state_dir: format!("{temp_dir}/state"),
            downloads_dir: format!("{temp_dir}/downloads"),
            file_browser_root_dir: temp_dir.to_string(),
            keyring: Arc::new(NoopKeyring),
        };
        start_server(options).expect("must start server");

        thread::sleep(time::Duration::from_secs(1));

        stop_server().expect("must stop server");
    }
}
