use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DomainError {
    ProtocolError,
    IntegrityError,
    NotFound(String),
    IoError(String),
    ParseError(String),
}

impl From<std::io::Error> for DomainError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => DomainError::NotFound(error.to_string()),
            _ => DomainError::IoError(error.to_string()),
        }
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(error: serde_json::Error) -> Self {
        DomainError::ParseError(error.to_string())
    }
}
