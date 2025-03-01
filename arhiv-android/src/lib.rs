use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use anyhow::{anyhow, ensure, Context, Result};
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use tokio::runtime::Runtime;

use arhiv::{ArhivOptions, ArhivServer};
use rs_utils::log;

static RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| Mutex::new(None));
static ARHIV_SERVER: LazyLock<Mutex<Option<ArhivServer>>> = LazyLock::new(|| Mutex::new(None));

fn start_server(options: ArhivOptions) -> Result<String> {
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

    Ok(server_info.ui_url)
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

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer(
    mut env: JNIEnv,
    _class: JClass,
    app_files_dir: JString,
    external_storage_dir: JString,
) -> jstring {
    log::setup_android_logger("me.mbsoftware.arhiv");

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

    let options = ArhivOptions::new_android(app_files_dir, external_storage_dir);
    let url = start_server(options).expect("must start server");
    log::info!("Started server: {url}");

    let output = env.new_string(url).expect("Couldn't create java string!");

    output.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_stopServer() {
    stop_server().expect("must stop server");
    log::info!("Stopped server");
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use arhiv::ArhivOptions;
    use rs_utils::TempFile;

    use crate::{start_server, stop_server};

    #[test]
    fn test_arhiv_server_for_android() {
        let temp_dir = TempFile::new_with_details("AndroidTest", "");
        temp_dir.mkdir().expect("must create temp dir");

        let options = ArhivOptions {
            storage_dir: format!("{temp_dir}/storage"),
            state_dir: format!("{temp_dir}/state"),
            file_browser_root_dir: temp_dir.to_string(),
        };
        start_server(options).expect("must start server");

        thread::sleep(time::Duration::from_secs(1));

        stop_server().expect("must stop server");
    }
}
