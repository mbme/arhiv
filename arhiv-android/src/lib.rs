// use tokio::runtime::Runtime;

use jni::{
    objects::{JClass, JString},
    JNIEnv,
};

use arhiv::{Arhiv, ArhivOptions};
use rs_utils::log;

fn get_root_dir(files_dir: &str) -> String {
    if cfg!(feature = "production-mode") {
        format!("{files_dir}/arhiv")
    } else {
        format!("{files_dir}/arhiv-debug")
    }
}

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer(
    mut env: JNIEnv,
    _class: JClass,
    files_dir: JString,
) {
    log::setup_android_logger("me.mbsoftware.arhiv");

    let files_dir: String = env
        .get_string(&files_dir)
        .expect("Must read JNI string")
        .into();
    log::debug!("Files dir: {files_dir}");

    let arhiv = Arhiv::open(
        get_root_dir(&files_dir),
        ArhivOptions {
            auto_commit: true,
            discover_peers: true,
            ..Default::default()
        },
    );

    if let Err(err) = arhiv {
        log::error!("ERR: {err}");
    }

    // let rt = Runtime::new().expect("failed to create tokio runtime");
    // rt.block_on(async {
    //     arhiv.start_server().await.expect("failed to start server");
    // });
}
