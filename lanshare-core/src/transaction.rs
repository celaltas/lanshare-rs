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
}

impl Transaction {
    pub fn new(filename: &str, temp_dir: &Path, final_dir: &Path) -> io::Result<Self> {
        let tx_id = format!("tx_{}", Uuid::new_v4());
        let tmp_dir = temp_dir.join(&tx_id);
        std::fs::create_dir_all(&tmp_dir)?;

        let file_path = tmp_dir.join(format!("{}.part", filename));
        let final_path = final_dir.join(filename);
        let file = File::create(&file_path)?;

        Ok(Self {
            id: tx_id,
            tmp_path: file_path,
            final_path,
            file,
        })
    }
    pub fn writer(&mut self) -> &mut File {
        &mut self.file
    }
    pub fn commit(&mut self) -> io::Result<()> {
        self.file.flush()?;
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
