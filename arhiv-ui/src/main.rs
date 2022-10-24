use rs_utils::log::setup_logger;

use arhiv_ui::start_ui_server;

#[tokio::main]
async fn main() {
    setup_logger();
    start_ui_server().await;
}
