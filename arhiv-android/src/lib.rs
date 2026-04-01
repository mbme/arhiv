mod keyring;

use std::{
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, anyhow, ensure};
use jni::{
    EnvUnowned, jni_sig, jni_str,
    objects::{JClass, JObject, JString, JValue},
};
use tokio::runtime::Runtime;

use arhiv::{Arhiv, ArhivOptions, ArhivServer, ServerInfo};
use rs_utils::{SecretString, init_global_rayon_threadpool, log};

use self::keyring::AndroidKeyring;

static LOG_INITIALIZED: AtomicBool = AtomicBool::new(false);

static RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| Mutex::new(None));
static ARHIV_SERVER: LazyLock<Mutex<Option<ArhivServer>>> = LazyLock::new(|| Mutex::new(None));

fn start_server(options: ArhivOptions, port: u16) -> Result<ServerInfo> {
    let mut runtime_lock = RUNTIME
        .lock()
        .map_err(|err| anyhow!("Failed to lock RUNTIME: {err}"))?;
    ensure!(runtime_lock.is_none(), "Runtime already started");

    let mut server_lock = ARHIV_SERVER
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV_SERVER: {err}"))?;
    ensure!(server_lock.is_none(), "Server already started");

    let worker_threads_count = Arhiv::optimal_number_of_worker_threads();
    log::debug!("Using {worker_threads_count} worker threads");

    init_global_rayon_threadpool(worker_threads_count)?;

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(worker_threads_count);
    builder.enable_all();
    let runtime = builder.build().context("Failed to create tokio runtime")?;

    let server = runtime.block_on(ArhivServer::start(options, port))?;
    let server_info = server.get_info().clone();

    if cfg!(test) {
        server.arhiv.baza.create("test1234".into())?;
    }

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

#[unsafe(no_mangle)]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass,
    app_files_dir: JString,
    external_storage_dir: JString,
    downloads_dir: JString,
    password: JString,
    android_controller: JObject, // AndroidController
) -> JObject<'local> {
    let outcome = env.with_env(|env| -> jni::errors::Result<JObject<'local>> {
        // the function might be called multiple times, if android app was unloaded in background
        if LOG_INITIALIZED.load(Ordering::SeqCst) {
            log::info!("Logger already initialized");
        } else {
            log::setup_android_logger("me.mbsoftware.arhiv");
            log::setup_panic_hook();
            LOG_INITIALIZED.store(true, Ordering::SeqCst);

            log::debug!("Initialized logger and panic hook");
        }

        let app_files_dir: String = app_files_dir.to_string();
        log::debug!("Files dir: {app_files_dir}");

        let external_storage_dir: String = external_storage_dir.to_string();
        log::debug!("Storage dir: {external_storage_dir}");

        let downloads_dir: String = downloads_dir.to_string();
        log::debug!("Donwloads dir: {downloads_dir}");

        let password: Option<SecretString> = if password.as_raw().is_null() {
            log::debug!("No password");
            None
        } else {
            log::debug!("Got password");
            let password: String = password.to_string();

            Some(password.into())
        };

        let android_controller = env
            .new_global_ref(android_controller)
            .expect("Must turn AndroidController instance into global ref");
        let jvm = env.get_java_vm().expect("Can't get reference to JVM");
        let options = ArhivOptions {
            storage_dir: format!("{external_storage_dir}/Arhiv"),
            state_dir: app_files_dir,
            downloads_dir,
            file_browser_root_dir: external_storage_dir,
            keyring: AndroidKeyring::new_arhiv_keyring(password, android_controller, jvm),
        };

        let server_info =
            start_server(options, ArhivServer::DEFAULT_PORT).expect("must start server");

        // Create an instance of me.mbsoftware.arhiv.ServerInfo using JNI
        let server_info_class = env
            .find_class(jni_str!("me/mbsoftware/arhiv/ServerInfo"))
            .expect("Couldn't find ServerInfo class");
        let server_info_object = env
            .alloc_object(&server_info_class)
            .expect("Couldn't allocate ServerInfo object");

        let ui_url = env
            .new_string(server_info.ui_url)
            .expect("Couldn't create java String!");
        env.set_field(
            &server_info_object,
            jni_str!("uiUrl"),
            jni_sig!("Ljava/lang/String;"),
            JValue::from(&ui_url),
        )
        .expect("Couldn't set field String uiUrl");

        let auth_token = env
            .new_string(server_info.auth_token)
            .expect("Couldn't create java String!");
        env.set_field(
            &server_info_object,
            jni_str!("authToken"),
            jni_sig!("Ljava/lang/String;"),
            JValue::from(&auth_token),
        )
        .expect("Couldn't set field String authToken");

        let certificate = env
            .byte_array_from_slice(&server_info.certificate)
            .expect("Couldn't create java byte[]!");
        env.set_field(
            &server_info_object,
            jni_str!("certificate"),
            jni_sig!("[B"),
            JValue::from(&certificate),
        )
        .expect("Couldn't set field byte[] certificate");

        Ok(server_info_object)
    });

    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_stopServer() {
    stop_server().expect("must stop server");
    log::info!("Stopped server");
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use arhiv::{ArhivKeyring, ArhivOptions};
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
            keyring: ArhivKeyring::new_noop(),
        };
        start_server(options, 0).expect("must start server");

        thread::sleep(time::Duration::from_secs(1));

        stop_server().expect("must stop server");
    }
}
