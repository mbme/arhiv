use std::{
    net::IpAddr,
    sync::{Mutex, OnceLock},
};

use anyhow::{ensure, Context, Result};
use mdns_sd::{Error as MDNSError, ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::broadcast;

use crate::get_hostname;

pub type InstanceName = String;

#[derive(Clone, Debug)]
pub enum MDNSEvent {
    InstanceDiscovered(PeerInfo),
    InstanceDisappeared(InstanceName),
}

pub struct MDNSService {
    mdns: OnceLock<ServiceDaemon>,
    service_name: String,
    instance_name: InstanceName,
    started: bool,
    server_fullname: Mutex<Option<String>>,
    client_started: Mutex<bool>,
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

        Ok(MDNSService {
            mdns: Default::default(),
            service_name,
            instance_name,
            started: true,
            client_started: Mutex::new(false),
            server_fullname: Mutex::new(None),
            events: broadcast::channel(42),
        })
    }

    fn get_mdns_service(&self) -> &ServiceDaemon {
        self.mdns
            .get_or_init(|| ServiceDaemon::new().expect("Failed to create daemon"))
    }

    pub fn get_events(&self) -> broadcast::Receiver<MDNSEvent> {
        self.events.0.subscribe()
    }

    fn get_service_type(&self) -> String {
        format!("{}._sub._http._tcp.local.", self.service_name)
    }

    pub fn start_server(&self, port: u16) -> Result<()> {
        ensure!(self.started, "MDNS service must be started");

        let mut server_fullname = self.server_fullname.try_lock().expect("must lock");
        ensure!(server_fullname.is_none(), "MDNS server already started");

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

        server_fullname.replace(my_service.get_fullname().to_string());

        let mdns = self.get_mdns_service();
        mdns.register(my_service)
            .context("Failed to register service")?;

        log::debug!("Started MDNS server for instance {}", self.instance_name);

        Ok(())
    }

    pub fn stop_server(&self) {
        let mut server_fullname = self.server_fullname.lock().expect("must lock");

        if let Some(fullname) = server_fullname.take() {
            let mdns = self.get_mdns_service();

            loop {
                match mdns.unregister(&fullname) {
                    Ok(channel) => {
                        let status = channel.recv().expect("must read status");
                        log::debug!(
                            "Stopped MDNS server for instance {}: {:?}",
                            self.instance_name,
                            status,
                        );
                        break;
                    }
                    Err(MDNSError::Again) => {}
                    Err(err) => {
                        log::error!(
                            "Error while stopping MDNS server for instance {}: {err}",
                            self.instance_name
                        );
                        break;
                    }
                }
            }

            *server_fullname = None;
        }
    }

    pub fn start_client(&self) -> Result<()> {
        ensure!(self.started, "MDNS service must be started");

        let mut client_started = self.client_started.try_lock().expect("must lock");
        ensure!(!*client_started, "MDNS client already started");

        let mdns = self.get_mdns_service();
        let receiver = mdns
            .browse(&self.get_service_type())
            .context("Failed to browse")?;

        *client_started = true;
        log::debug!("Started MDNS client for instance {}", self.instance_name);

        let local_instance_name = self.instance_name.clone();

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
                            instance_name: instance_name.clone(),
                            ips: info.get_addresses().iter().cloned().collect(),
                            port: info.get_port(),
                        };

                        if peer_info.ips.is_empty() {
                            log::debug!(
                                "No known IP addresses for instance {instance_name}, ignoring"
                            );
                            continue;
                        }

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

    pub fn stop_client(&self) {
        let mut client_started = self.client_started.try_lock().expect("must lock");
        if !*client_started {
            return;
        }

        let mdns = match self.mdns.get() {
            Some(mdns) => mdns,
            None => {
                return;
            }
        };

        loop {
            match mdns.stop_browse(&self.get_service_type()) {
                Ok(_) => {
                    log::debug!("Stopped MDNS client for instance {}", self.instance_name);
                    break;
                }
                Err(MDNSError::Again) => {}
                Err(err) => {
                    log::error!("Failed to stop MDNS client: {err}");
                    break;
                }
            }
        }

        *client_started = false;
    }

    pub fn shutdown(&mut self) {
        if !self.started {
            return;
        }

        // self.stop_client();
        // self.stop_server();

        let mdns = match self.mdns.get() {
            Some(mdns) => mdns,
            None => {
                return;
            }
        };

        loop {
            match mdns.shutdown() {
                Ok(res) => {
                    let status = res.recv().expect("must read shutdown status");
                    self.started = false;
                    log::debug!("Stopped MDNS service {}: {:?}", self.service_name, status);
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

fn extract_instance_name_from_fullname(fullname: &str) -> String {
    fullname
        .split('.')
        .next()
        .map(ToString::to_string)
        .expect("failed to extract instance name")
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub instance_name: String,
    pub ips: Vec<IpAddr>,
    pub port: u16,
}
