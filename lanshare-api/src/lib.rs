use serde::{Deserialize, Serialize};

pub trait DiscoveryApi {
    fn list_peers(&self) -> Result<Vec<PeerApi>, ApiError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerApi {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub last_seen: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiError {
    Timeout,
    NetworkError(String),
    ServiceUnavailable,
}
