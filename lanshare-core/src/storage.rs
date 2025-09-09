use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

use crate::transaction::Transaction;

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
        expected_sha: [u8; 32],
    ) -> io::Result<Transaction> {
        Transaction::new(filename, expected_sha, &self.tmp_dir, &self.final_dir)
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
