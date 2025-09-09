use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use uuid::Uuid;

pub struct Transaction {
    id: String,
    tmp_path: PathBuf,
    final_path: PathBuf,
    file: File,
    hasher: Sha256,
    expected_sha: [u8; 32],
}

pub struct TransactionWriter<'a> {
    tx: &'a mut Transaction,
}

impl<'a> Write for TransactionWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx.hasher.update(buf);
        self.tx.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tx.file.flush()
    }
}

impl Transaction {
    pub fn new(
        filename: &str,
        expected_sha: [u8; 32],
        temp_dir: &Path,
        final_dir: &Path,
    ) -> io::Result<Self> {
        let tx_id = format!("tx_{}", Uuid::new_v4());
        let tmp_dir = temp_dir.join(&tx_id);
        std::fs::create_dir_all(&tmp_dir)?;

        let file_path = tmp_dir.join(format!("{}.part", filename));
        let final_path = final_dir.join(filename);
        let file = File::create(&file_path)?;
        let hasher = Sha256::new();

        Ok(Self {
            id: tx_id,
            tmp_path: file_path,
            final_path,
            file,
            expected_sha,
            hasher,
        })
    }
    pub fn writer(&mut self) -> TransactionWriter<'_> {
        TransactionWriter { tx: self }
    }
    pub fn commit(&mut self) -> io::Result<()> {
        let actual_sha = self.hasher.clone().finalize();
        if actual_sha[..] != self.expected_sha[..] {
            self.rollback()?;
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SHA mismatch"));
        }
        std::fs::rename(&self.tmp_path, &self.final_path)?;

        if let Some(parent_dir) = self.tmp_path.parent() {
            std::fs::remove_dir_all(parent_dir)
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path has no parent directory",
            ));
        }
    }
    pub fn rollback(&self) -> io::Result<()> {
        if let Some(parent_dir) = self.tmp_path.parent() {
            std::fs::remove_dir_all(parent_dir)
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path has no parent directory",
            ));
        }
    }
}
