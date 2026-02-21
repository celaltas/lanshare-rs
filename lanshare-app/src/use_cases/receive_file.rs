use lanshare_domain::{DomainError, FileBlock, FileManifest, StoragePort};

pub struct ReceiveFileUseCase<S: StoragePort> {
    storage: S,
}

impl<S: StoragePort> ReceiveFileUseCase<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn accept_transfer(&self, manifest: &FileManifest) -> Result<u64, DomainError> {
        self.storage.prepare_for_receive(manifest)?;
        self.storage.get_written_bytes(&manifest.file_id)
    }

    pub fn process_chunk(&self, block: &FileBlock) -> Result<(), DomainError> {
        self.storage.write_block(block)
    }

    pub fn finish_transfer(&self, file_id: &str) -> Result<(), DomainError> {
        self.storage.complete_transfer(file_id)
    }
}
