use std::{collections::HashSet, net::Ipv4Addr};

use anyhow::{ensure, Context, Result};
use mdns_sd::{Error as MDNSError, ServiceDaemon, ServiceEvent, ServiceInfo};

pub struct MDNSNode {
    mdns: ServiceDaemon,
    service_name: String,
}

pub struct MDNSServer<'m> {
    fullname: String,
    mdns: &'m ServiceDaemon,
}

impl<'m> MDNSServer<'m> {
    pub fn get_instance_name(&self) -> String {
        extract_instance_name_from_fullname(&self.fullname)
    }

    pub fn stop(self) {
        loop {
            match self.mdns.unregister(&self.fullname) {
                Ok(channel) => {
                    log::info!(
                        "Stopped MDNS service for instance {}: {:?}",
                        self.get_instance_name(),
                        channel.recv().expect("must read result"),
                    );
                    return;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!(
                        "Error while stopping MDNS service for instance {}: {err}",
                        self.get_instance_name(),
                    );
                    return;
                }
            }
        }
    }
}

pub struct MDNSClient<'m> {
    service_type: String,
    mdns: &'m ServiceDaemon,
}

impl<'m> MDNSClient<'m> {
    pub fn stop(self) {
        loop {
            match self.mdns.stop_browse(&self.service_type) {
                Ok(_) => {
                    log::info!("Stopped MDNS client");
                    return;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!("Failed to stop MDNS client: {err}");
                    return;
                }
            }
        }
    }
}

impl MDNSNode {
    pub fn new(service_name: impl Into<String>) -> Result<Self> {
        let service_name = service_name.into();

        ensure!(
            service_name.chars().next() == Some('_'),
            "service_name must start with an underscore"
        );

        let mdns = ServiceDaemon::new().context("Failed to create daemon")?;

        Ok(MDNSNode { mdns, service_name })
    }

    fn get_service_type(&self) -> String {
        format!("{}._sub._http._tcp.local.", self.service_name)
    }

    pub fn start_server(&self, instance_name: &str, port: u16) -> Result<MDNSServer> {
        // Create a service info.
        let host_name = "mb-host-name.local"; // FIXME

        let my_service = ServiceInfo::new(
            &self.get_service_type(),
            instance_name,
            host_name,
            "", // auto-discover is enabled
            port,
            None,
        )
        .context("Failed to construct service info")?
        .enable_addr_auto();

        let fullname = my_service.get_fullname().to_string();

        self.mdns
            .register(my_service)
            .context("Failed to register service")?;

        log::info!("started MDNS service for instance {instance_name}");

        Ok(MDNSServer {
            fullname,
            mdns: &self.mdns,
        })
    }

    pub fn start_client<F: Fn(MDNSEvent) -> () + Send + 'static>(
        &self,
        handler: F,
    ) -> Result<MDNSClient> {
        let receiver = self
            .mdns
            .browse(&self.get_service_type())
            .context("Failed to browse")?;

        log::info!("started MDNS client");

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let instance_name =
                            extract_instance_name_from_fullname(info.get_fullname());

                        log::info!("Registered an instance: {instance_name}");

                        handler(MDNSEvent::ServiceDiscovered {
                            ips: info.get_addresses().clone(),
                            port: info.get_port(),
                            instance_name,
                        });
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let instance_name = extract_instance_name_from_fullname(&fullname);

                        log::info!("Unregistered an instance: {instance_name}");

                        handler(MDNSEvent::ServiceDisappeared { instance_name });
                    }
                    other_event => {
                        log::debug!("Received other event: {:?}", &other_event);
                    }
                }
            }
        });

        Ok(MDNSClient {
            mdns: &self.mdns,
            service_type: self.get_service_type(),
        })
    }

    pub fn shutdown(self) {
        loop {
            match self.mdns.shutdown() {
                Ok(_) => {
                    log::info!("Stopped MDNS node for service {}", self.service_name,);
                    return;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!(
                        "Error while stopping MDNS node for service {}: {err}",
                        self.service_name,
                    );
                    return;
                }
            }
        }
    }
}

fn extract_instance_name_from_fullname(fullname: &str) -> String {
    fullname
        .split('.')
        .next()
        .map(ToString::to_string)
        .expect("failed to extract instance name")
}

#[derive(Debug)]
pub enum MDNSEvent {
    ServiceDiscovered {
        ips: HashSet<Ipv4Addr>,
        port: u16,
        instance_name: String,
    },
    ServiceDisappeared {
        instance_name: String,
    },
}

impl MDNSEvent {
    pub fn get_instance_name(&self) -> &str {
        match self {
            MDNSEvent::ServiceDiscovered { instance_name, .. } => &instance_name,
            MDNSEvent::ServiceDisappeared { instance_name, .. } => &instance_name,
        }
    }
}
