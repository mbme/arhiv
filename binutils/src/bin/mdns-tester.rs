use anyhow::Result;
use tokio::signal;

use rs_utils::{
    generate_random_id,
    log::setup_logger,
    mdns::{MDNSEvent, MDNSService},
};

#[tokio::main]
pub async fn main() -> Result<()> {
    setup_logger();

    let instance_name = generate_random_id();

    let mut service = MDNSService::new("_mdns-tester", instance_name)?;
    service.start_client()?;

    let mut rx = service.get_events();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(MDNSEvent::InstanceDiscovered(peer_info)) => {
                    println!("discovered {:#?}", peer_info);
                }
                Ok(MDNSEvent::InstanceDisappeared(instance_name)) => {
                    println!("lost instance {instance_name}");
                }
                Err(err) => {
                    eprintln!("got error: {err}");
                    break;
                }
            }
        }
    });

    let mut server = service.start_server(9999)?;

    signal::ctrl_c().await.expect("failed to listen for event");

    server.stop();

    Ok(())
}
