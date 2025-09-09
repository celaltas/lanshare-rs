use std::io::{self, Read, Write};

pub struct MessageHeader {
    pub name: String,
    pub size: u64,
    pub sha256: [u8; 32],
}

impl MessageHeader {
    pub fn read_from<T: Read>(stream: &mut T) -> io::Result<Self> {
        let mut name_buf = [0u8; 256];
        stream.read_exact(&mut name_buf)?;
        let name = String::from_utf8_lossy(&name_buf)
            .trim_end_matches(char::from(0))
            .to_string();
        let mut size_buf = [0u8; 8];
        stream.read_exact(&mut size_buf)?;
        let size = u64::from_le_bytes(size_buf);
        let mut hash_buf = [0u8; 32];
        stream.read_exact(&mut hash_buf)?;
        Ok(MessageHeader {
            name,
            size,
            sha256: hash_buf,
        })
    }
    pub fn write_to<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        let mut name_buf = [0u8; 256];
        let name_bytes = self.name.as_bytes();
        let name_len = name_bytes.len().min(256);
        name_buf[..name_len].copy_from_slice(&name_bytes[..name_len]);
        stream.write_all(&name_buf)?;
        let size_buf = self.size.to_le_bytes();
        stream.write_all(&size_buf)?;
        stream.write_all(&self.sha256)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    #[test]
    fn test_message_header_serialization() {
        use super::MessageHeader;
        use std::io::Cursor;

        let mut hasher = Sha256::new();
        hasher.update(b"hello world");
        let hash_bytes = hasher.finalize();
        let as_array: [u8; 32] = hash_bytes.into();

        let header = MessageHeader {
            name: "test_file.txt".to_string(),
            size: 12345,
            sha256: as_array,
        };

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let deserialized_header = MessageHeader::read_from(&mut cursor).unwrap();

        assert_eq!(header.name, deserialized_header.name);
        assert_eq!(header.size, deserialized_header.size);
        assert_eq!(header.sha256, deserialized_header.sha256);
    }
}
