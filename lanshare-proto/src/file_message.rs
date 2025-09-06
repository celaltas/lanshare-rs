use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use crate::MessageHeader;

pub struct FileMessage {
    pub header: MessageHeader,
    pub content: Vec<u8>,
}

pub fn read_file_message<R: Read>(stream: &mut R) -> io::Result<FileMessage> {
    let header = MessageHeader::read_from(stream)?;
    let mut content = vec![0u8; header.size as usize];
    stream.read_exact(&mut content)?;
    Ok(FileMessage { header, content })
}

pub fn send_file_message<W: Write>(stream: &mut W, path: &Path) -> io::Result<()> {
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
    header.write_to(stream)?;
    let mut reader = io::BufReader::new(file);
    io::copy(&mut reader, stream)?;
    stream.flush()?;
    Ok(())
}
