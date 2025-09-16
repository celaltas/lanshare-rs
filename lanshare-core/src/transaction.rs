use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransactionMeta {
    pub id: String,
    pub filename: String,
    pub tmp_path: String,
    pub final_path: String,
    pub expected_sha: String,
    pub written_bytes: u64,
    pub total_size: u64,
}

impl TransactionMeta {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let meta: Self = serde_json::from_str(&json)?;
        Ok(meta)
    }
}

pub struct Transaction {
    pub id: String,
    pub tmp_path: PathBuf,
    pub final_path: PathBuf,
    pub file: File,
    pub hasher: Sha256,
    pub expected_sha: [u8; 32],
    pub written_bytes: u64,
    pub total_size: u64,
    pub meta_path: PathBuf,
}

impl From<&Transaction> for TransactionMeta {
    fn from(value: &Transaction) -> Self {
        let filename = value
            .final_path
            .file_name()
            .unwrap()
            .to_str()
            .map(From::from)
            .unwrap();
        let hex_string = value
            .expected_sha
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        Self {
            id: value.id.clone(),
            filename,
            tmp_path: value.tmp_path.to_string_lossy().to_string(),
            final_path: value.final_path.to_string_lossy().to_string(),
            expected_sha: hex_string,
            written_bytes: value.written_bytes,
            total_size: value.total_size,
        }
    }
}

pub struct TransactionWriter<'a> {
    tx: &'a mut Transaction,
}

impl<'a> Write for TransactionWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.tx.file.write(buf)?;
        self.tx.hasher.update(&buf[..n]);
        self.tx.written_bytes += n as u64;

        if self.tx.written_bytes % (1024 * 1024) < n as u64
            || self.tx.written_bytes == self.tx.total_size
        {
            self.tx.persist_meta()?;
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tx.persist_meta()?;
        self.tx.file.flush()
    }
}

impl Transaction {
    pub fn new(
        filename: &str,
        expected_sha: [u8; 32],
        total_size: u64,
        temp_dir: &Path,
        final_dir: &Path,
    ) -> Result<Transaction, std::io::Error> {
        let tx_id = format!("tx_{}", Uuid::new_v4());
        let tmp_dir = temp_dir.join(&tx_id);
        std::fs::create_dir_all(&tmp_dir)?;

        let meta_path = tmp_dir.join(format!("{}.meta", filename));
        let file_path = tmp_dir.join(format!("{}.part", filename));
        let final_path = final_dir.join(filename);
        let file = File::create(&file_path)?;
        let hasher = Sha256::new();
        let hex_string = expected_sha
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        let meta = TransactionMeta {
            id: tx_id.clone(),
            filename: filename.to_owned(),
            tmp_path: tmp_dir.to_string_lossy().to_string(),
            final_path: final_path.to_string_lossy().to_string(),
            expected_sha: hex_string,
            written_bytes: 0,
            total_size,
        };

        meta.save(&meta_path)?;

        Ok(Self {
            id: tx_id,
            tmp_path: file_path,
            final_path,
            file,
            expected_sha,
            hasher,
            written_bytes: 0,
            total_size,
            meta_path,
        })
    }
    pub fn writer(&mut self) -> TransactionWriter<'_> {
        TransactionWriter { tx: self }
    }
    pub fn commit(&mut self) -> io::Result<()> {
        if self.written_bytes < self.total_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Upload incomplete: {} of {} bytes written",
                    self.written_bytes, self.total_size
                ),
            ));
        }
        let actual_sha = self.hasher.clone().finalize();
        if actual_sha[..] != self.expected_sha[..] {
            self.rollback()?;
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SHA mismatch"));
        }
        std::fs::rename(&self.tmp_path, &self.final_path)?;
        if let Some(parent_dir) = self.tmp_path.parent() {
            std::fs::remove_dir_all(parent_dir)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path has no parent directory",
            ))
        }
    }

    pub fn resume(meta: TransactionMeta) -> io::Result<Self> {
        let tmp_dir = PathBuf::from(&meta.tmp_path);
        let file_path = tmp_dir.join(format!("{}.part", meta.filename));
        let final_path = PathBuf::from(&meta.final_path);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&file_path)?;

        let mut hasher = Sha256::new();
        if meta.written_bytes > 0 {
            let mut f = File::open(&file_path)?;
            let mut buffer = vec![0u8; 8192];
            let mut remaining = meta.written_bytes;
            while remaining > 0 {
                let to_read = std::cmp::min(remaining, buffer.len() as u64) as usize;
                let n = f.read(&mut buffer[..to_read])?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
                remaining -= n as u64;
            }
            file.seek(SeekFrom::Start(meta.written_bytes))?;
        }

        let mut expected_sha = [0u8; 32];

        for (i, chunk) in meta.expected_sha.as_bytes().chunks(2).enumerate() {
            let byte_str = std::str::from_utf8(chunk)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8 in hex"))?;
            expected_sha[i] = u8::from_str_radix(byte_str, 16)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid hex digit"))?;
        }

        Ok(Self {
            id: meta.id,
            tmp_path: file_path,
            final_path,
            file,
            expected_sha,
            hasher,
            written_bytes: meta.written_bytes,
            total_size: meta.total_size,
            meta_path: tmp_dir.join(format!("{}.meta", meta.filename)),
        })
    }

    pub fn rollback(&self) -> io::Result<()> {
        if let Some(parent_dir) = self.tmp_path.parent() {
            std::fs::remove_dir_all(parent_dir)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path has no parent directory",
            ))
        }
    }
    pub fn persist_meta(&self) -> io::Result<()> {
        let meta = TransactionMeta::from(self);
        meta.save(&self.meta_path)
    }
}
