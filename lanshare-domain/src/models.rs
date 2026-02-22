use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub address: SocketAddr,
    pub last_seen: u64,
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
