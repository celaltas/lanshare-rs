pub enum LanShareMessage {
    TransferRequest(TransferRequestPayload),
    TransferResponse(TransferResponsePayload),
    DataChunk(DataChunkPayload),
    Error(ErrorPayload),
}

pub struct TransferRequestPayload {
    pub name: String,
    pub size: u64,
    pub sha256: [u8; 32],
}

pub struct TransferResponsePayload {
    pub accepted: bool,
    pub resume_offset: u64,
}
pub struct DataChunkPayload {
    pub offset: u64,
    pub data: Vec<u8>,
}
pub struct ErrorPayload {
    pub message: String,
}
