use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub address: SocketAddr,
    pub last_seen: u64,
}

impl Peer {
    pub fn new(name: String, address: SocketAddr, last_seen: u64) -> Self {
        Peer {
            name,
            address,
            last_seen,
        }
    }
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileManifest {
    pub file_id: String,
    pub name: String,
    pub size: u64,
    pub sha256: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct FileBlock {
    pub file_id: String,
    pub offset: u64,
    pub data: Vec<u8>,
}
