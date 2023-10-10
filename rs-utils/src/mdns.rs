use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::{ensure, Context, Result};
use mdns_sd::{Error as MDNSError, ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::{broadcast, watch};

use crate::get_hostname;

pub type InstanceName = String;
pub type Peers = HashMap<InstanceName, PeerInfo>;

#[derive(Clone, Debug)]
pub enum MDNSEvent {
    InstanceDiscovered(PeerInfo),
    InstanceDisappeared(InstanceName),
}

pub struct MDNSService {
    mdns: ServiceDaemon,
    service_name: String,
    instance_name: InstanceName,
    started: bool,
    peers: Option<watch::Receiver<Peers>>,
    events: (broadcast::Sender<MDNSEvent>, broadcast::Receiver<MDNSEvent>),
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

        let mut service = MDNSService {
            mdns,
            service_name,
            instance_name,
            started: true,
            peers: Default::default(),
            events: broadcast::channel(42),
        };

        service.start_client()?;

        Ok(service)
    }

    pub fn get_peers_rx(&self) -> watch::Receiver<Peers> {
        self.peers
            .as_ref()
            .expect("peers must be initialized")
            .clone()
    }

    pub fn get_events(&self) -> broadcast::Receiver<MDNSEvent> {
        self.events.0.subscribe()
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

        log::debug!("Started MDNS server for instance {}", self.instance_name);

        Ok(MDNSServer {
            fullname,
            mdns: &self.mdns,
            started: true,
        })
    }

    fn start_client(&mut self) -> Result<()> {
        ensure!(self.started, "MDNS service must be started");

        let receiver = self
            .mdns
            .browse(&self.get_service_type())
            .context("Failed to browse")?;

        log::debug!("Started MDNS client for instance {}", self.instance_name);

        let local_instance_name = self.instance_name.clone();

        let (tx, rx) = watch::channel::<Peers>(HashMap::new());
        self.peers = Some(rx);

        let events = self.events.0.clone();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let instance_name =
                            extract_instance_name_from_fullname(info.get_fullname());

                        if instance_name == local_instance_name {
                            continue;
                        }

                        log::debug!("Registered an instance: {instance_name}");

                        let peer_info = PeerInfo {
                            ips: info.get_addresses().iter().cloned().collect(),
                            port: info.get_port(),
                        };

                        tx.send_modify(|peers| {
                            peers.insert(instance_name, peer_info.clone());
                        });

                        events
                            .send(MDNSEvent::InstanceDiscovered(peer_info))
                            .expect("must send MDNS event");
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        let instance_name = extract_instance_name_from_fullname(&fullname);

                        if instance_name == local_instance_name {
                            continue;
                        }

                        log::debug!("Unregistered an instance: {instance_name}");

                        tx.send_modify(|peers| {
                            peers.remove(&instance_name);
                        });

                        events
                            .send(MDNSEvent::InstanceDisappeared(instance_name))
                            .expect("must send MDNS event");
                    }
                    other_event => {
                        log::trace!("Received other event: {:?}", &other_event);
                    }
                }
            }
        });

        Ok(())
    }

    fn stop_client(&self) {
        loop {
            match self.mdns.stop_browse(&self.get_service_type()) {
                Ok(_) => {
                    log::debug!("Stopped MDNS client for instance {}", self.instance_name);
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

    pub fn shutdown(&mut self) {
        if !self.started {
            return;
        }

        self.stop_client();

        loop {
            match self.mdns.shutdown() {
                Ok(_) => {
                    self.started = false;
                    log::debug!("Stopped MDNS service {}", self.service_name,);
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

                    log::debug!(
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

fn extract_instance_name_from_fullname(fullname: &str) -> String {
    fullname
        .split('.')
        .next()
        .map(ToString::to_string)
        .expect("failed to extract instance name")
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub ips: Vec<Ipv4Addr>,
    pub port: u16,
}
