use std::process::Command;

use rs_utils::log::setup_logger;
use server::start_server;

mod commander;
mod server;

#[tokio::main]
async fn main() {
    setup_logger();

    let (join_handle, addr) = start_server().await;

    Command::new("chromium")
        .arg(format!("http://{}", addr))
        .spawn()
        .expect("failed to run chromium");

    join_handle.await.expect("must join");
}
