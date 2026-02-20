use std::io;

#[derive(Debug)]
pub enum ProtoError {
    Io(io::Error),
    InvalidPrefix([u8; 2]),
    InvalidData(String),
    InvalidMessage,
}

impl From<io::Error> for ProtoError {
    fn from(err: io::Error) -> Self {
        ProtoError::Io(err)
    }
}
