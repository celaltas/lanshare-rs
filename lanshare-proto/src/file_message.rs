use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

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
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?
            .to_string();
        let size = metadata.len();
        let header = MessageHeader {
            name: filename,
            size,
        };
        header.write_to(&mut stream)?;
        let mut reader = io::BufReader::new(file);
        io::copy(&mut reader, &mut stream)?;
        stream.flush()?;
        Ok(())
    }
}
