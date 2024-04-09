use std::sync::Arc;

use anyhow::Result;

use rs_utils::{
    log,
    mdns::{MDNSEvent, MDNSService},
};
use tokio::task::JoinHandle;

use crate::{entities::InstanceId, Baza, DEBUG_MODE};

use super::SyncManager;

pub struct MDNSDiscoveryService {
    mdns_service: MDNSService,
}

impl MDNSDiscoveryService {
    pub fn new(baza: &Baza) -> Result<MDNSDiscoveryService> {
        let conn = baza.get_connection()?;

        let instance_id = conn.get_instance_id()?;

        let login = conn.get_login()?;

        let app_name = baza.get_app_name();

        let mut service_name = format!("_{login}@{app_name}");
        if DEBUG_MODE {
            service_name.push_str("-debug");
        }

        let mdns_service = MDNSService::new(service_name, instance_id)?;

        Ok(MDNSDiscoveryService { mdns_service })
    }

    pub fn start_mdns_server(&self, port: u16) -> Result<()> {
        self.mdns_service.start_server(port)
    }

    pub fn start_mdns_client(&self, sync_manager: Arc<SyncManager>) -> Result<JoinHandle<()>> {
        self.mdns_service.start_client()?;

        let mut mdns_events = self.mdns_service.get_events();
        let task = tokio::spawn(async move {
            loop {
                match mdns_events.recv().await {
                    Ok(mdns_event) => match mdns_event {
                        MDNSEvent::InstanceDiscovered(peer_info) => {
                            let instance_id = InstanceId::from_string(peer_info.instance_name);

                            let ip_address = peer_info.ips[0];

                            let address = format!("https://{ip_address}:{}", peer_info.port);

                            if let Err(err) =
                                sync_manager.add_network_agent(instance_id.clone(), &address)
                            {
                                log::error!(
                                    "Failed to add network agent {instance_id} {address}: {err}"
                                );
                            }
                        }
                        MDNSEvent::InstanceDisappeared(instance_name) => {
                            let instance_id = InstanceId::from_string(instance_name);

                            sync_manager.remove_agent(&instance_id);
                        }
                    },
                    Err(err) => log::error!("Failed to receive MDNS event: {err}"),
                }

                log::debug!("MDNS client task ended");
            }
        });

        Ok(task)
    }
}
