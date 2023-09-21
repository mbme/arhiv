use std::net::Ipv4Addr;

use anyhow::{ensure, Context, Result};
use mdns_sd::{Error as MDNSError, ServiceDaemon, ServiceEvent, ServiceInfo};

use crate::get_hostname;

pub struct MDNSService {
    mdns: ServiceDaemon,
    service_name: String,
    instance_name: String,
    started: bool,
}

impl MDNSService {
    pub fn new(service_name: impl Into<String>, instance_name: impl Into<String>) -> Result<Self> {
        let service_name = service_name.into();
        let instance_name = instance_name.into();

        ensure!(
            service_name.starts_with('_'),
            "service_name must start with an underscore"
        );

        let mdns = ServiceDaemon::new().context("Failed to create daemon")?;

        Ok(MDNSService {
            mdns,
            service_name,
            instance_name,
            started: true,
        })
    }

    fn get_service_type(&self) -> String {
        format!("{}._sub._http._tcp.local.", self.service_name)
    }

    pub fn start_server(&self, port: u16) -> Result<MDNSServer> {
        ensure!(self.started, "MDNS service must be started");

        let host_name = get_hostname()?;

        let my_service = ServiceInfo::new(
            &self.get_service_type(),
            &self.instance_name,
            &host_name,
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

        log::info!("started MDNS server for instance {}", self.instance_name);

        Ok(MDNSServer {
            fullname,
            mdns: &self.mdns,
            started: true,
        })
    }

    pub fn start_client<F: Fn(MDNSEvent) + Send + 'static>(
        &self,
        handler: F,
    ) -> Result<MDNSClient> {
        ensure!(self.started, "MDNS service must be started");

        let receiver = self
            .mdns
            .browse(&self.get_service_type())
            .context("Failed to browse")?;

        log::info!("started MDNS client for instance {}", self.instance_name);

        let local_instance_name = self.instance_name.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let instance_name =
                            extract_instance_name_from_fullname(info.get_fullname());

                        if instance_name == local_instance_name {
                            continue;
                        }

                        log::info!("Registered an instance: {instance_name}");

                        handler(MDNSEvent::ServiceDiscovered {
                            ips: info.get_addresses().iter().cloned().collect(),
                            port: info.get_port(),
                            instance_name,
                        });
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let instance_name = extract_instance_name_from_fullname(&fullname);

                        if instance_name == local_instance_name {
                            continue;
                        }

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
            started: true,
        })
    }

    pub fn shutdown(&mut self) {
        if !self.started {
            return;
        }

        loop {
            match self.mdns.shutdown() {
                Ok(_) => {
                    self.started = false;
                    log::info!("Stopped MDNS service {}", self.service_name,);
                    return;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!(
                        "Error while stopping MDNS service {}: {err}",
                        self.service_name,
                    );
                    return;
                }
            }
        }
    }
}

impl Drop for MDNSService {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub struct MDNSServer<'m> {
    fullname: String,
    mdns: &'m ServiceDaemon,
    started: bool,
}

impl<'m> MDNSServer<'m> {
    pub fn get_instance_name(&self) -> String {
        extract_instance_name_from_fullname(&self.fullname)
    }

    pub fn stop(&mut self) {
        if !self.started {
            return;
        }

        loop {
            match self.mdns.unregister(&self.fullname) {
                Ok(channel) => {
                    self.started = false;

                    log::info!(
                        "Stopped MDNS server for instance {}: {:?}",
                        self.get_instance_name(),
                        channel.recv().expect("must read result"),
                    );
                    return;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!(
                        "Error while stopping MDNS server for instance {}: {err}",
                        self.get_instance_name(),
                    );
                    return;
                }
            }
        }
    }
}

impl<'m> Drop for MDNSServer<'m> {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct MDNSClient<'m> {
    service_type: String,
    mdns: &'m ServiceDaemon,
    started: bool,
}

impl<'m> MDNSClient<'m> {
    pub fn stop(&mut self) {
        if !self.started {
            return;
        }

        loop {
            match self.mdns.stop_browse(&self.service_type) {
                Ok(_) => {
                    self.started = false;
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

impl<'m> Drop for MDNSClient<'m> {
    fn drop(&mut self) {
        self.stop();
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
        ips: Vec<Ipv4Addr>,
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
            MDNSEvent::ServiceDiscovered { instance_name, .. } => instance_name,
            MDNSEvent::ServiceDisappeared { instance_name, .. } => instance_name,
        }
    }
}
