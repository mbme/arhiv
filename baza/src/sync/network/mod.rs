use anyhow::{Context, Result};

use rs_utils::mdns::MDNSService;

mod rpc_client;
mod rpc_server;

pub use rpc_client::BazaClient;
pub use rpc_server::{build_rpc_router, respond_with_blob, start_rpc_server};

use crate::Baza;

impl Baza {
    pub fn init_mdns_service(&self) -> Result<MDNSService> {
        let instance_id = self
            .get_connection()
            .and_then(|conn| conn.get_instance_id())
            .context("failed to read instance_id")?;

        let app_name = self.get_name();

        MDNSService::new(format!("_{app_name}-baza"), instance_id)
    }
}
