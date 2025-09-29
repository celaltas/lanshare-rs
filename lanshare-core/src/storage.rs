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
        println!("checking transaction whether already exist...");
        let meta = self.find_existing_meta(filename);
        let meta = meta?;
        if let Some(tm) = meta {
            println!("resuming found for {}", filename);
            Transaction::resume(tm)
        } else {
            println!("transaction not found, create new one");
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Transaction not found",
            ))
        }
    }

    fn find_existing_meta(&self, filename: &str) -> io::Result<Option<TransactionMeta>> {
        if !self.tmp_dir.exists() {
            return Ok(None);
        }

        let entries: Vec<_> = fs::read_dir(&self.tmp_dir)?.collect();

        for entry in entries.into_iter() {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let transaction_dir = entry.path();
                let tmp_path = transaction_dir.join(format!("{}.part", filename));
                let meta_path = transaction_dir.join(format!("{}.meta", filename));
                if meta_path.exists() && tmp_path.exists() {
                    let meta = TransactionMeta::load(&meta_path)?;
                    return Ok(Some(meta));
                }
            }
        }
        Ok(None)
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
    #[test]
    fn test_find_existing_meta() -> io::Result<()> {
        let project_root = std::env::current_dir()?.parent().unwrap().to_path_buf();
        let storage_tmp = project_root.join("storage");
        let fs = FileStorage::new(&storage_tmp).unwrap();
        let result = fs.find_existing_meta("test2.docx")?;
        println!("res: {:#?}", result);

        Ok(())
    }
}
