use anyhow::Result;
use tokio::signal;

use rs_utils::{generate_random_id, log::setup_logger, mdns::MDNSNode};

#[tokio::main]
pub async fn main() -> Result<()> {
    setup_logger();

    let node = MDNSNode::new("_mdns-tester")?;

    let instance_name = generate_random_id();

    let server = node.start_server(&instance_name, 9999)?;
    let client = node.start_client(move |event| {
        if event.get_instance_name() != instance_name {
            println!("Event: {:#?}", event);
        }
    })?;

    signal::ctrl_c().await.expect("failed to listen for event");

    client.stop();
    server.stop();
    node.shutdown();

    Ok(())
}
