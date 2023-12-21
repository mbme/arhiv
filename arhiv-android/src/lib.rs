// use tokio::runtime::Runtime;

use arhiv::{Arhiv, ArhivOptions};
use rs_utils::log;

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer() {
    log::setup_android_logger("me.mbsoftware.arhiv");

    log::debug!("HELLO WORLD!");
    let arhiv = Arhiv::open_with_options(ArhivOptions {
        auto_commit: true,
        discover_peers: true,
        ..Default::default()
    });

    if let Err(err) = arhiv {
        log::error!("ERR: {err}");
    }

    // let rt = Runtime::new().expect("failed to create tokio runtime");
    // rt.block_on(async {
    //     arhiv.start_server().await.expect("failed to start server");
    // });
}
