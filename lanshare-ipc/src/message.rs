use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SuccessMessage<T> {
    pub id: u64,
    pub status: String,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub id: u64,
    pub status: String,
    pub error: String,
    pub code: String,
}


#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandRequest {
    ListPeers {
        id: Option<u64>,
    },
    SendFile {
        id: Option<u64>,
        path: String,
        peer: String,
        file_name: Option<String>,
    },
    GetStatus {
        id: Option<u64>,
    },
    CancelTransfer {
        id: Option<u64>,
        transfer_id: String,
    },
}