use std::{sync::Mutex, time::Duration};

use anyhow::{anyhow, ensure, Context, Result};
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use lazy_static::lazy_static;
use tokio::runtime::Runtime;

use arhiv::{Arhiv, ArhivConfigExt, ArhivOptions, ArhivServer};
use rs_utils::{dir_exists, log};

lazy_static! {
    static ref RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);
    static ref ARHIV_SERVER: Mutex<Option<ArhivServer>> = Mutex::new(None);
}

fn get_root_dir(files_dir: &str) -> String {
    if cfg!(feature = "production-mode") {
        format!("{files_dir}/arhiv")
    } else {
        format!("{files_dir}/arhiv-debug")
    }
}

fn start_server(files_dir: &str) -> Result<String> {
    let mut runtime_lock = RUNTIME
        .lock()
        .map_err(|err| anyhow!("Failed to lock RUNTIME: {err}"))?;
    ensure!(runtime_lock.is_none(), "Runtime already started");

    let mut server_lock = ARHIV_SERVER
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV_SERVER: {err}"))?;
    ensure!(server_lock.is_none(), "Server already started");

    let root_dir = get_root_dir(files_dir);
    let root_dir_exists = dir_exists(&root_dir)?;

    let runtime = Runtime::new().context("failed to create tokio runtime")?;

    let _guard = runtime.enter();

    let arhiv = {
        if cfg!(test) {
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    create: true,
                    auto_commit: false,
                    discover_peers: false,
                    mdns_server: false,
                },
            )?;

            let tx = arhiv.baza.get_tx()?;
            tx.set_server_port(0)?;
            tx.commit()?;

            arhiv
        } else {
            Arhiv::open(
                root_dir,
                ArhivOptions {
                    create: !root_dir_exists,
                    auto_commit: true,
                    discover_peers: true,
                    mdns_server: true,
                },
            )?
        }
    };

    let server = runtime.block_on(ArhivServer::start(arhiv))?;
    let ui_url = server.get_ui_url();

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

    runtime.block_on(server.stop())?;
    runtime.shutdown_timeout(Duration::from_millis(500));

    Ok(())
}

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer(
    mut env: JNIEnv,
    _class: JClass,
    files_dir: JString,
) -> jstring {
    log::setup_android_logger("me.mbsoftware.arhiv");

    let files_dir: String = env
        .get_string(&files_dir)
        .expect("Must read JNI string")
        .into();
    log::debug!("Files dir: {files_dir}");

    let url = start_server(&files_dir).expect("must start server");
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

        start_server(temp_dir.as_ref()).expect("must start server");

        thread::sleep(time::Duration::from_secs(1));

        stop_server().expect("must stop server");
    }
}
