use anyhow::{Context, Result};

use rs_utils::mdns::MDNSService;

mod rpc_client;
mod rpc_server;

pub use rpc_client::BazaClient;
pub use rpc_server::{build_rpc_router, respond_with_blob};

use crate::Baza;

use super::DEBUG_MODE;

impl Baza {
    pub fn init_mdns_service(&self) -> Result<MDNSService> {
        let instance_id = self
            .get_connection()
            .and_then(|conn| conn.get_instance_id())
            .context("failed to read instance_id")?;

        let app_name = self.get_name();

        let mut service_name = format!("_{app_name}-baza");
        if DEBUG_MODE {
            service_name.push_str("-debug");
        }

        MDNSService::new(service_name, instance_id)
    }
}
