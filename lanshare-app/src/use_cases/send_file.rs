use lanshare_domain::{
    error::DomainError,
    models::Peer,
    ports::{NetworkPort, StoragePort},
};
use lanshare_proto::{
    codec::encode_message,
    messages::{DataChunkPayload, LanShareMessage, TransferRequestPayload},
};

pub struct SendFileUseCase<S: StoragePort, N: NetworkPort> {
    storage: S,
    network: N,
}

impl<S: StoragePort, N: NetworkPort> SendFileUseCase<S, N> {
    pub fn new(storage: S, network: N) -> Self {
        Self { storage, network }
    }

    pub fn execute(&self, peer: &Peer, file_path: &str) -> Result<(), DomainError> {
        let manifest = self.storage.create_file_manifest(file_path)?;
        let mut connection = self.network.connect(peer)?;

        let request_payload = TransferRequestPayload {
            name: manifest.name.clone(),
            size: manifest.size,
            sha256: manifest.sha256,
        };

        let request_msg = LanShareMessage::TransferRequest(request_payload);

        let mut request_buffer = Vec::new();
        encode_message(&mut request_buffer, &request_msg)
            .map_err(|_e| DomainError::ProtocolError)?;
        connection.send(&request_buffer)?;

        let chunk_size: usize = 8192;
        let mut offset: u64 = 0;
        let mut chunk_buffer = Vec::with_capacity(chunk_size + 128);
        while offset < manifest.size {
            chunk_buffer.clear();
            let block = self.storage.read_block(file_path, offset, chunk_size)?;
            let chunk_payload = DataChunkPayload {
                offset: block.offset,
                data: block.data,
            };
            let chunk_msg = LanShareMessage::DataChunk(chunk_payload);

            encode_message(&mut chunk_buffer, &chunk_msg)
                .map_err(|_e| DomainError::ProtocolError)?;
            connection.send(&request_buffer)?;
            offset += chunk_size as u64;
        }

        Ok(())
    }
}
