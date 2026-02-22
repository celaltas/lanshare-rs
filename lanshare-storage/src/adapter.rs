use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use lanshare_domain::{
    error::DomainError,
    models::{FileBlock, FileManifest},
    ports::StoragePort,
};

use crate::{
    hash::{compute_file_sha256, sha_to_hex},
    transaction::TransactionMeta,
};

pub struct LocalFileSystemAdapter {
    base_dir: PathBuf,
    tmp_dir: PathBuf,
    final_dir: PathBuf,
}

impl LocalFileSystemAdapter {
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
}

impl StoragePort for LocalFileSystemAdapter {
    fn create_file_manifest(&self, file_path: &str) -> Result<FileManifest, DomainError> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let size = metadata.len();

        let name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.bin")
            .to_string();

        let file_id = uuid::Uuid::new_v4().to_string();
        let sha256 = compute_file_sha256(file_path)?;

        Ok(FileManifest {
            file_id,
            name,
            size,
            sha256,
        })
    }

    fn prepare_for_receive(&self, manifest: &FileManifest) -> Result<(), DomainError> {
        let meta = TransactionMeta {
            id: manifest.file_id.clone(),
            filename: manifest.name.clone(),
            expected_sha: sha_to_hex(&manifest.sha256),
            written_bytes: 0,
            total_size: manifest.size,
        };

        let meta_path = self.tmp_dir.join(format!("{}.meta", manifest.file_id));
        let meta_json = serde_json::to_string_pretty(&meta)?;
        fs::write(meta_path, meta_json)?;

        let part_path = self.tmp_dir.join(format!("{}.part", manifest.file_id));
        File::create(part_path)?;
        Ok(())
    }

    fn get_written_bytes(&self, file_id: &str) -> Result<u64, DomainError> {
        let meta_path = self.tmp_dir.join(format!("{}.meta", file_id));
        if meta_path.exists() {
            let json = fs::read_to_string(meta_path)?;
            let meta: TransactionMeta = serde_json::from_str(&json)?;
            Ok(meta.written_bytes)
        } else {
            Ok(0)
        }
    }

    fn read_block(
        &self,
        file_path: &str,
        offset: u64,
        length: usize,
    ) -> Result<FileBlock, DomainError> {
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0; length];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        Ok(FileBlock {
            file_id: file_path.to_string(),
            offset,
            data: buffer,
        })
    }

    fn write_block(&self, block: &FileBlock) -> Result<(), DomainError> {
        let part_path = self.tmp_dir.join(format!("{}.part", block.file_id));
        let meta_path = self.tmp_dir.join(format!("{}.meta", block.file_id));

        let mut file = fs::OpenOptions::new().write(true).open(part_path)?;
        file.seek(SeekFrom::Start(block.offset))?;
        file.write_all(&block.data)?;

        if meta_path.exists() {
            let json = fs::read_to_string(&meta_path)?;
            let mut meta: TransactionMeta = serde_json::from_str(&json).unwrap();
            meta.written_bytes += block.data.len() as u64;
            fs::write(meta_path, serde_json::to_string(&meta).unwrap())?;
        }
        Ok(())
    }

    fn complete_transfer(&self, file_id: &str) -> Result<(), DomainError> {
        let part_path = self.tmp_dir.join(format!("{}.part", file_id));
        let meta_path = self.tmp_dir.join(format!("{}.meta", file_id));

        let json = fs::read_to_string(&meta_path)?;
        let meta: TransactionMeta = serde_json::from_str(&json).unwrap();

        let actual_sha = compute_file_sha256(&part_path)?;
        if sha_to_hex(&actual_sha) != meta.expected_sha {
            return Err(DomainError::IntegrityError);
        }

        let final_path = self.final_dir.join(meta.filename);
        fs::rename(part_path, final_path)?;
        let _ = fs::remove_file(meta_path);
        Ok(())
    }

    fn cancel_transfer(&self, file_id: &str) -> Result<(), DomainError> {
        let part_path = self.tmp_dir.join(format!("{}.part", file_id));
        let meta_path = self.tmp_dir.join(format!("{}.meta", file_id));

        let _ = fs::remove_file(part_path);
        let _ = fs::remove_file(meta_path);
        Ok(())
    }
}
