use crate::{
    error::DomainError,
    models::{FileBlock, FileManifest, Peer},
};

pub trait StoragePort: Send + Sync {
    fn create_file_manifest(&self, file_path: &str) -> Result<FileManifest, DomainError>;
    fn prepare_for_receive(&self, manifest: &FileManifest) -> Result<(), DomainError>;
    fn get_written_bytes(&self, file_id: &str) -> Result<u64, DomainError>;
    fn read_block(
        &self,
        file_path: &str,
        offset: u64,
        length: usize,
    ) -> Result<FileBlock, DomainError>;
    fn write_block(&self, block: &FileBlock) -> Result<(), DomainError>;
    fn complete_transfer(&self, file_id: &str) -> Result<(), DomainError>;
    fn cancel_transfer(&self, file_id: &str) -> Result<(), DomainError>;
}

pub trait NetworkPort: Send + Sync {
    fn send_data(&self, peer: &Peer, data: &[u8]) -> Result<(), DomainError>;
}

pub trait DiscoveryPort: Send + Sync {
    fn discover_peers(&self) -> Result<Vec<Peer>, DomainError>;
    fn broadcast_presence(&self, peer: &Peer) -> Result<(), DomainError>;
}
