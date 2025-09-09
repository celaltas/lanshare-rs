use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::MessageHeader;

pub struct FileMessage;

impl FileMessage {
    pub fn receive<R: Read, W: Write>(stream: &mut R, writer: &mut W, size: u64) -> io::Result<()> {
        let mut remaining = size;
        let mut buffer = [0u8; 8192];
        while remaining > 0 {
            let n = stream.read(&mut buffer)?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Client disconnected",
                ));
            }
            writer.write_all(&buffer[..n])?;
            remaining -= n as u64;
        }
        Ok(())
    }

    pub fn send<W: Write>(mut stream: W, path: &Path) -> io::Result<()> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?
            .to_string();

        let sha256: [u8; 32] = {
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 8192];
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            hasher.finalize().into()
        };

        file.seek(io::SeekFrom::Start(0))?;

        let size = metadata.len();
        let header = MessageHeader {
            name: filename,
            size,
            sha256,
        };

        header.write_to(&mut stream)?;

        io::copy(&mut file, &mut stream)?;
        stream.flush()?;
        Ok(())
    }

    fn hash_file<T: Read>(mut reader: T) -> io::Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        let result = hasher.finalize();
        Ok(result.into())
    }
}
