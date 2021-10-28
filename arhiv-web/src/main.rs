use rs_utils::log::setup_logger;

#[tokio::main]
async fn main() {
    setup_logger();
    println!("TEST");
}
