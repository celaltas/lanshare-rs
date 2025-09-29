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
                break;
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

        let sha256: [u8; 32] = FileMessage::hash_file(&mut file)?;
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
    pub fn send_partial<W: Write>(mut stream: W, path: &Path, cutoff: u64) -> io::Result<()> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?
            .to_string();

        let sha256: [u8; 32] = FileMessage::hash_file(&mut file)?;
        file.seek(io::SeekFrom::Start(0))?;

        let size = metadata.len();
        let header = MessageHeader {
            name: filename,
            size,
            sha256,
        };

        header.write_to(&mut stream)?;

        let mut buf = vec![0u8; 8192];
        let mut written = 0;
        while written < cutoff {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let to_write = std::cmp::min(n as u64, cutoff - written) as usize;
            stream.write_all(&buf[..to_write])?;
            written += to_write as u64;
        }
        Ok(())
    }

    fn hash_file<T: Read>(reader: &mut T) -> io::Result<[u8; 32]> {
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

    pub fn skip_bytes<R: Read>(stream: &mut R, bytes_to_skip: u64) -> io::Result<()> {
        let mut remaining = bytes_to_skip;
        let mut buffer = [0u8; 8192]; 

        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buffer.len() as u64) as usize;
            let n = stream.read(&mut buffer[..to_read])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "Stream ended while skipping bytes: {} bytes remaining",
                        remaining
                    ),
                ));
            }
            remaining -= n as u64;
        }
        println!("Successfully skipped {} bytes", bytes_to_skip);
        Ok(())
    }
}
