// use tokio::runtime::Runtime;

use arhiv::{Arhiv, ArhivOptions};

#[no_mangle]
pub extern "C" fn Java_me_mbsoftware_arhiv_ArhivServer_startServer() {
    eprintln!("HELLO WORLD!");
    let arhiv = Arhiv::open_with_options(ArhivOptions {
        auto_commit: true,
        discover_peers: true,
        ..Default::default()
    });

    if arhiv.is_err() {
        eprintln!("ERR");
    }

    // let rt = Runtime::new().expect("failed to create tokio runtime");
    // rt.block_on(async {
    //     arhiv.start_server().await.expect("failed to start server");
    // });
}
