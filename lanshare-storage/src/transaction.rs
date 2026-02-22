use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMeta {
    pub id: String,
    pub filename: String,
    pub expected_sha: String,
    pub written_bytes: u64,
    pub total_size: u64,
}
