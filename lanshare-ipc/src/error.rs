#[derive(Debug)]
pub enum IPCError {
    ConnectionRefused,
    InvalidJson,
    UnknownCommand,
    PeerNotFound,
    FileNotFound,
    TransferFailed,
    Other(String),
}
