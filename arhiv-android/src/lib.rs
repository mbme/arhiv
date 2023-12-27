use std::sync::Mutex;

use anyhow::{anyhow, bail, ensure, Result};
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
    static ref RUNTIME: Runtime = Runtime::new().expect("failed to create tokio runtime");
    static ref ARHIV: Mutex<Option<ArhivServer>> = Mutex::new(None);
}

fn get_root_dir(files_dir: &str) -> String {
    if cfg!(feature = "production-mode") {
        format!("{files_dir}/arhiv")
    } else {
        format!("{files_dir}/arhiv-debug")
    }
}

fn start_server(files_dir: &str) -> Result<String> {
    let mut server = ARHIV
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV: {err}"))?;
    ensure!(server.is_none(), "server already started");

    let root_dir = get_root_dir(files_dir);
    let root_dir_exists = dir_exists(&root_dir)?;

    let _guard = RUNTIME.enter();

    let arhiv = {
        if cfg!(test) {
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    create: true,
                    auto_commit: false,
                    discover_peers: false,
                },
            )?;

            let tx = arhiv.baza.get_tx()?;
            tx.set_server_port(0)?;
            tx.commit()?;

            arhiv
        } else {
            let arhiv = Arhiv::open(
                root_dir,
                ArhivOptions {
                    create: !root_dir_exists,
                    auto_commit: true,
                    discover_peers: true,
                },
            )?;

            arhiv
        }
    };

    let instance = ArhivServer::start(arhiv)?;
    let ui_url = instance.get_ui_url();
    *server = Some(instance);

    Ok(ui_url)
}

fn stop_server() -> Result<()> {
    let mut lock = ARHIV
        .lock()
        .map_err(|err| anyhow!("Failed to lock ARHIV: {err}"))?;

    let server = lock.take();

    if let Some(server) = server {
        RUNTIME.block_on(server.stop())?;
    } else {
        bail!("Server not started");
    }

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
