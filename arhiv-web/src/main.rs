use arhiv_ui3::start_ui_server;
use rs_utils::log::setup_logger;

#[tokio::main]
async fn main() {
    setup_logger();
    start_ui_server().await;
}
