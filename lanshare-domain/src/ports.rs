use crate::{
    error::DomainError,
    models::{FileBlock, FileManifest, Peer},
};
use std::sync::Arc;

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

impl<T: StoragePort + ?Sized> StoragePort for Arc<T> {
    fn create_file_manifest(&self, file_path: &str) -> Result<FileManifest, DomainError> {
        (**self).create_file_manifest(file_path)
    }
    fn prepare_for_receive(&self, manifest: &FileManifest) -> Result<(), DomainError> {
        (**self).prepare_for_receive(manifest)
    }
    fn get_written_bytes(&self, file_id: &str) -> Result<u64, DomainError> {
        (**self).get_written_bytes(file_id)
    }
    fn read_block(
        &self,
        file_path: &str,
        offset: u64,
        length: usize,
    ) -> Result<FileBlock, DomainError> {
        (**self).read_block(file_path, offset, length)
    }
    fn write_block(&self, block: &FileBlock) -> Result<(), DomainError> {
        (**self).write_block(block)
    }
    fn complete_transfer(&self, file_id: &str) -> Result<(), DomainError> {
        (**self).complete_transfer(file_id)
    }
    fn cancel_transfer(&self, file_id: &str) -> Result<(), DomainError> {
        (**self).cancel_transfer(file_id)
    }
}

pub trait NetworkConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), DomainError>;
}

pub trait NetworkPort: Send + Sync {
    fn connect(&self, peer: &Peer) -> Result<Box<dyn NetworkConnection>, DomainError>;
}

impl<T: NetworkPort + ?Sized> NetworkPort for Arc<T> {
    fn connect(&self, peer: &Peer) -> Result<Box<dyn NetworkConnection>, DomainError> {
        (**self).connect(peer)
    }
}

pub trait DiscoveryPort: Send + Sync {
    fn discover_peers(&self) -> Result<Vec<Peer>, DomainError>;
    fn broadcast_presence(&self, peer: &Peer) -> Result<(), DomainError>;
}
