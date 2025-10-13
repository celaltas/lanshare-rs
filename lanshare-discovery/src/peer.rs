use std::{
    net::IpAddr,
    time::{Duration, SystemTime},
};

use mdns_sd::ResolvedService;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    pub last_seen: SystemTime,
}

impl From<Box<ResolvedService>> for PeerInfo {
    fn from(service: Box<ResolvedService>) -> Self {
        let addrs = service.get_addresses_v4().iter().copied().next();
        Self {
            id: service.get_fullname().to_owned(),
            name: service.get_fullname().to_owned(),
            ip: addrs.unwrap().into(),
            port: service.get_port(),
            last_seen: SystemTime::now(),
        }
    }
}

impl PeerInfo {
    pub fn new(id: String, name: String, ip: IpAddr, port: u16) -> Self {
        Self {
            id,
            name,
            ip,
            port,
            last_seen: SystemTime::now(),
        }
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.last_seen)
            .map(|t| t > timeout)
            .unwrap_or(true)
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now()
    }
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
