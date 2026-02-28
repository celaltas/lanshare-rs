use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, RwLock},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use lanshare_domain::{error::DomainError, models::Peer, ports::DiscoveryPort};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

const SERVICE_NAME: &str = "_lanshare._tcp.local.";

pub struct MdnsDiscoveryAdapter {
    daemon: ServiceDaemon,
    registry: Arc<RwLock<HashMap<String, Peer>>>,
}

impl MdnsDiscoveryAdapter {
    pub fn new() -> Result<Self, DomainError> {
        let daemon = ServiceDaemon::new().map_err(|e| DomainError::IoError(e.to_string()))?;
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let registry_clone = Arc::clone(&registry);

        let browse_rx = daemon
            .browse(SERVICE_NAME)
            .map_err(|e| DomainError::IoError(e.to_string()))?;

        thread::spawn(move || {
            while let Ok(event) = browse_rx.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        if let Some(ip_v4) = info.get_addresses_v4().iter().next() {
                            let addr = SocketAddr::new(IpAddr::V4(*ip_v4), info.get_port());
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();

                            let peer = Peer::new(info.get_fullname().to_string(), addr, now);

                            if let Ok(mut guard) = registry_clone.write() {
                                guard.insert(info.get_fullname().to_string(), peer);
                            }
                        }
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        if let Ok(mut guard) = registry_clone.write() {
                            guard.remove(&fullname);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self { daemon, registry })
    }
}

impl DiscoveryPort for MdnsDiscoveryAdapter {
    fn discover_peers(&self) -> Result<Vec<Peer>, DomainError> {
        let mut guard = self
            .registry
            .write()
            .map_err(|_| DomainError::IoError("Lock failed".into()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        guard.retain(|_, peer| now - peer.last_seen < 60);

        Ok(guard.values().cloned().collect())
    }

    fn broadcast_presence(&self, peer: &Peer) -> Result<(), DomainError> {
        let instance_name = peer.name.clone();
        let host_name = format!("{}.local.", instance_name);
        let ip = peer.address.ip().to_string();
        let port = peer.address.port();
        let properties = [("version", "1.0")];

        let service_info = ServiceInfo::new(
            SERVICE_NAME,
            &instance_name,
            &host_name,
            ip,
            port,
            &properties[..],
        )
        .map_err(|e| DomainError::IoError(e.to_string()))?;

        self.daemon
            .register(service_info)
            .map_err(|e| DomainError::IoError(e.to_string()))?;

        Ok(())
    }
}
