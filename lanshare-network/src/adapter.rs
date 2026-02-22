use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use lanshare_app::use_cases::receive_file::ReceiveFileUseCase;
use lanshare_domain::{
    error::DomainError,
    models::{FileBlock, FileManifest, Peer},
    ports::{NetworkConnection, NetworkPort, StoragePort},
};
use lanshare_proto::{
    codec::{decode_message, encode_message},
    messages::{ErrorPayload, LanShareMessage, TransferResponsePayload},
};
use uuid::Uuid;

pub struct TCPConnection {
    socket: TcpStream,
}

impl NetworkConnection for TCPConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), DomainError> {
        self.socket.write_all(data)?;
        Ok(())
    }
}

pub struct TcpNetworkAdapter {}

impl NetworkPort for TcpNetworkAdapter {
    fn connect(&self, peer: &Peer) -> Result<Box<dyn NetworkConnection>, DomainError> {
        let socket = TcpStream::connect(peer.address())?;
        Ok(Box::new(TCPConnection { socket }))
    }
}

impl TcpNetworkAdapter {
    pub fn start_listening<S: StoragePort + 'static>(
        use_case: Arc<ReceiveFileUseCase<S>>,
    ) -> Result<(), DomainError> {
        let listener =
            TcpListener::bind("0.0.0.0:8080").map_err(|e| DomainError::IoError(e.to_string()))?;
        println!("Listening on 0.0.0.0:8080...");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    let use_case_clone = Arc::clone(&use_case);

                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, use_case_clone) {
                            eprintln!("Connection error: {:?}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }
        Ok(())
    }
}

fn handle_connection<S: StoragePort>(
    mut stream: TcpStream,
    use_case: Arc<ReceiveFileUseCase<S>>,
) -> Result<(), DomainError> {
    let first_msg = decode_message(&mut stream).map_err(|_| DomainError::ProtocolError)?;
    let mut current_file_id = String::new();

    if let LanShareMessage::TransferRequest(payload) = first_msg {
        let local_file_id = Uuid::new_v4().to_string();
        current_file_id = local_file_id.clone();

        let manifest = FileManifest {
            file_id: local_file_id,
            name: payload.name,
            size: payload.size,
            sha256: payload.sha256,
        };

        let written_bytes = use_case.accept_transfer(&manifest)?;

        let response = LanShareMessage::TransferResponse(TransferResponsePayload {
            accepted: true,
            resume_offset: written_bytes,
        });

        let mut buffer = Vec::new();
        encode_message(&mut buffer, &response).map_err(|_| DomainError::ProtocolError)?;
        stream
            .write_all(&buffer)
            .map_err(|e| DomainError::IoError(e.to_string()))?;
    } else {
        send_error_to_peer(
            &mut stream,
            "Expected TransferRequest as the first message!",
        );
        return Err(DomainError::ProtocolError);
    }

    loop {
        match decode_message(&mut stream) {
            Ok(LanShareMessage::DataChunk(payload)) => {
                let block = FileBlock {
                    file_id: current_file_id.clone(),
                    offset: payload.offset,
                    data: payload.data,
                };
                use_case.process_chunk(&block)?;
            }
            Ok(LanShareMessage::TransferRequest(_)) | Ok(LanShareMessage::TransferResponse(_)) => {
                send_error_to_peer(
                    &mut stream,
                    "Protocol violation: Expected DataChunk, but received another handshake message.",
                );
                return Err(DomainError::ProtocolError);
            }
            Ok(LanShareMessage::Error(err)) => {
                eprintln!("Peer sent an error: {}", err.message);
                return Err(DomainError::ProtocolError);
            }
            Err(_) => {
                break;
            }
        }
    }

    use_case.finish_transfer(&current_file_id)?;
    println!("File transfer completed: {}", current_file_id);
    Ok(())
}

fn send_error_to_peer(stream: &mut TcpStream, error_msg: &str) {
    let error_payload = LanShareMessage::Error(ErrorPayload {
        message: error_msg.to_string(),
    });
    let mut buffer = Vec::new();
    if encode_message(&mut buffer, &error_payload).is_ok() {
        let _ = stream.write_all(&buffer);
    }
}
