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

use arhiv::{Arhiv, ArhivOptions, ArhivServer, Credentials, ServerInfo};
use rs_utils::log;

static RUNTIME: LazyLock<Mutex<Option<Runtime>>> = LazyLock::new(|| Mutex::new(None));
static ARHIV_SERVER: LazyLock<Mutex<Option<ArhivServer>>> = LazyLock::new(|| Mutex::new(None));

fn get_root_dir(files_dir: &str) -> String {
    if cfg!(feature = "production-mode") {
        format!("{files_dir}/arhiv")
    } else {
        format!("{files_dir}/arhiv-debug")
    }
}

fn start_server(files_dir: &str, file_browser_root_dir: Option<String>) -> Result<String> {
    let mut runtime_lock = RUNTIME
        .lock()
        .map_err(|err| anyhow!("Failed to lock RUNTIME: {err}"))?;
    ensure!(runtime_lock.is_none(), "Runtime already started");

    let mut server_lock = ARHIV_SERVER
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV_SERVER: {err}"))?;
    ensure!(server_lock.is_none(), "Server already started");

    let root_dir = get_root_dir(files_dir);

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    let runtime = builder.build().context("failed to create tokio runtime")?;

    let _guard = runtime.enter();

    let arhiv_options = {
        if cfg!(test) {
            let auth = Credentials::new("test".to_string(), "test1234".into())?;

            Arhiv::create(root_dir.clone(), auth)?;

            ArhivOptions {
                file_browser_root_dir,
                ..Default::default()
            }
        } else {
            ArhivOptions {
                auto_commit: true,
                discover_peers: true,
                file_browser_root_dir,
            }
        }
    };

    let server = runtime.block_on(ArhivServer::start(&root_dir, arhiv_options, 0))?;
    let port = ServerInfo::get_server_port(&root_dir)?.context("Can't find server port")?;
    let ui_url = ServerInfo::get_ui_base_url(port);

    *server_lock = Some(server);
    *runtime_lock = Some(runtime);

    Ok(ui_url)
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
    files_dir: JString,
    storage_dir: JString,
) -> jstring {
    log::setup_android_logger("me.mbsoftware.arhiv");

    let files_dir: String = env
        .get_string(&files_dir)
        .expect("Must read JNI string files_dir")
        .into();
    log::debug!("Files dir: {files_dir}");

    let storage_dir: String = env
        .get_string(&storage_dir)
        .expect("Must read JNI string storage_dir")
        .into();
    log::debug!("Storage dir: {storage_dir}");

    let url = start_server(&files_dir, Some(storage_dir)).expect("must start server");
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

    use rs_utils::TempFile;

    use crate::{start_server, stop_server};

    #[test]
    fn test_arhiv_server_for_android() {
        let temp_dir = TempFile::new_with_details("AndroidTest", "");
        temp_dir.mkdir().expect("must create temp dir");

        start_server(temp_dir.as_ref(), None).expect("must start server");

        thread::sleep(time::Duration::from_secs(1));

        stop_server().expect("must stop server");
    }
}
