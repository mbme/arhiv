use anyhow::Result;
use tokio::signal;

use rs_utils::{generate_random_id, log::setup_logger, mdns::MDNSService};

#[tokio::main]
pub async fn main() -> Result<()> {
    setup_logger();

    let instance_name = generate_random_id();

    let service = MDNSService::new("_mdns-tester", instance_name)?;

    let mut server = service.start_server(9999)?;

    let mut client = service.start_client(|event| {
        println!("Event: {:#?}", event);
    })?;

    signal::ctrl_c().await.expect("failed to listen for event");

    client.stop();
    server.stop();

    Ok(())
}
