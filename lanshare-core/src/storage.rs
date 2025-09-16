use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

use crate::transaction::{Transaction, TransactionMeta};

#[derive(Debug)]
pub struct FileStorage {
    base_dir: PathBuf,
    tmp_dir: PathBuf,
    final_dir: PathBuf,
}

impl FileStorage {
    pub fn new(base_dir: impl AsRef<Path>) -> io::Result<Self> {
        let base_path = base_dir.as_ref().to_path_buf();
        let final_path = base_path.join("final");
        let temp_path = base_path.join("tmp");

        fs::create_dir_all(&final_path)?;
        fs::create_dir_all(&temp_path)?;

        Ok(Self {
            base_dir: base_path,
            tmp_dir: temp_path,
            final_dir: final_path,
        })
    }

    pub fn create_transaction(
        &self,
        filename: &str,
        total_size: u64,
        expected_sha: [u8; 32],
    ) -> io::Result<Transaction> {
        Transaction::new(
            filename,
            expected_sha,
            total_size,
            &self.tmp_dir,
            &self.final_dir,
        )
    }
    pub fn resume_transaction(&self, filename: &str) -> io::Result<Transaction> {
        let meta: TransactionMeta = self.find_existing_meta(filename)?.ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Transaction meta not found unexpectedly",
        ))?;

        Transaction::resume(meta)
    }

    fn find_existing_meta(&self, filename: &str) -> io::Result<Option<TransactionMeta>> {
        let tmp_path = self.tmp_dir.join(format!("{}.part", filename));
        let meta_path = self.tmp_dir.join(format!("{}.meta", filename));
        if meta_path.exists() && tmp_path.exists() {
            let meta = TransactionMeta::load(&meta_path)?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fs = FileStorage::new("/storage").unwrap();
        println!("fs: {:?}", fs)
    }
}
