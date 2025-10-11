use std::{
    net::IpAddr,
    time::{Duration, SystemTime},
};

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    pub last_seen: SystemTime,
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
