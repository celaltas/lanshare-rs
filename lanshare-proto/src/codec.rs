use std::io::{Read, Write};

use crate::{
    error::ProtoError,
    messages::{
        DataChunkPayload, ErrorPayload, LanShareMessage, TransferRequestPayload,
        TransferResponsePayload,
    },
};

pub fn encode_message<W: Write>(
    writer: &mut W,
    message: &LanShareMessage,
) -> Result<(), ProtoError> {
    match message {
        LanShareMessage::TransferRequest(TransferRequestPayload { name, size, sha256 }) => {
            writer.write_all(b"TQ")?;
            let mut name_buf = [0u8; 256];
            let name_bytes = name.as_bytes();
            let name_len = name_bytes.len().min(256);
            name_buf[..name_len].copy_from_slice(&name_bytes[..name_len]);
            writer.write_all(&name_buf)?;
            let size_buf = size.to_le_bytes();
            writer.write_all(&size_buf)?;
            writer.write_all(sha256)?;
            Ok(())
        }
        LanShareMessage::TransferResponse(TransferResponsePayload {
            accepted,
            resume_offset,
        }) => {
            writer.write_all(b"TR")?;
            writer.write_all(&[if *accepted { 1 } else { 0 }])?;
            writer.write_all(&resume_offset.to_le_bytes())?;
            Ok(())
        }
        LanShareMessage::DataChunk(DataChunkPayload { offset, data }) => {
            writer.write_all(b"DC")?;
            writer.write_all(&offset.to_le_bytes())?;
            let data_len = data.len() as u32;
            writer.write_all(&data_len.to_le_bytes())?;
            writer.write_all(data)?;
            Ok(())
        }
        LanShareMessage::Error(ErrorPayload { message }) => {
            writer.write_all(b"ER")?;
            let msg_bytes = message.as_bytes();
            writer.write_all(&(msg_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(msg_bytes)?;
            Ok(())
        }
    }
}

pub fn decode_message<R: Read>(reader: &mut R) -> Result<LanShareMessage, ProtoError> {
    let mut buffer = [0; 2];
    reader.read_exact(&mut buffer)?;

    match buffer {
        [b'T', b'Q'] => {
            let mut name_buf = [0u8; 256];
            reader.read_exact(&mut name_buf)?;
            let name = String::from_utf8_lossy(&name_buf)
                .trim_end_matches(char::from(0))
                .to_string();
            let mut size_buf = [0u8; 8];
            reader.read_exact(&mut size_buf)?;
            let size = u64::from_le_bytes(size_buf);
            let mut hash_buf = [0u8; 32];
            reader.read_exact(&mut hash_buf)?;
            Ok(LanShareMessage::TransferRequest(TransferRequestPayload {
                name,
                size,
                sha256: hash_buf,
            }))
        }
        [b'T', b'R'] => {
            let mut accepted_buf = [0u8; 1];
            reader.read_exact(&mut accepted_buf)?;
            let accepted = accepted_buf[0] != 0;
            let mut offset_buf = [0u8; 8];
            reader.read_exact(&mut offset_buf)?;
            let resume_offset = u64::from_le_bytes(offset_buf);

            Ok(LanShareMessage::TransferResponse(TransferResponsePayload {
                accepted,
                resume_offset,
            }))
        }
        [b'D', b'C'] => {
            let mut offset_buf = [0u8; 8];
            reader.read_exact(&mut offset_buf)?;
            let offset = u64::from_le_bytes(offset_buf);
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let data_len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; data_len];
            reader.read_exact(&mut data)?;
            Ok(LanShareMessage::DataChunk(DataChunkPayload {
                offset,
                data,
            }))
        }
        [b'E', b'R'] => {
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let msg_len = u32::from_le_bytes(len_buf) as usize;

            let mut msg_buf = vec![0u8; msg_len];
            reader.read_exact(&mut msg_buf)?;
            let message = String::from_utf8_lossy(&msg_buf).to_string();

            Ok(LanShareMessage::Error(ErrorPayload { message }))
        }
        _ => Err(ProtoError::InvalidMessage),
    }
}
