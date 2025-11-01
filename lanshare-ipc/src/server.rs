use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use serde::Serialize;

use crate::{
    error::IPCError,
    message::{CommandRequest, ErrorMessage, SuccessMessage},
};

#[derive(Debug)]
pub struct IPCServer {
    socket_path: Arc<PathBuf>,
    shutdown: Arc<AtomicBool>,
    listener_handle: Option<JoinHandle<()>>,
}

impl Clone for IPCServer {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            shutdown: self.shutdown.clone(),
            listener_handle: None,
        }
    }
}

impl IPCServer {
    pub fn new(path: PathBuf, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            socket_path: Arc::new(path),
            shutdown,
            listener_handle: None,
        }
    }
    pub fn start(&mut self) -> Result<(), IPCError> {
        self.initialize_socket()?;
        let self_clone = self.clone();
        let path = self.socket_path.to_path_buf();
        let shutdown = self.shutdown.clone();
        let handle = thread::spawn(move || self_clone.run_listener(shutdown, path));
        self.listener_handle = Some(handle);
        Ok(())
    }

    fn initialize_socket(&self) -> Result<(), IPCError> {
        fs::remove_file(self.socket_path.as_path())
            .map_err(|_e| IPCError::Other("Failed to remove socket".into()))?;
        Ok(())
    }

    fn bind_socket(&self) -> Result<UnixListener, IPCError> {
        let listener = UnixListener::bind(self.socket_path.as_path())
            .map_err(|_e| IPCError::Other("Failed to bind socket".into()))?;
        Ok(listener)
    }
    fn set_permissions(&self) -> Result<(), IPCError> {
        fs::set_permissions(
            self.socket_path.as_path(),
            fs::Permissions::from_mode(0o600),
        )
        .map_err(|_e| IPCError::Other("Failed to set permissions".into()))?;
        Ok(())
    }

    fn accept_connection(&self, listener: &UnixListener) -> Result<UnixStream, IPCError> {
        let (stream, addr) = listener
            .accept()
            .map_err(|_e| IPCError::Other("Failed to accept connection".into()))?;
        print!("accept a new client from {:#?}", addr);
        Ok(stream)
    }

    fn validate_connection(&self, _client_socket: &UnixStream) -> Result<bool, IPCError> {
        Ok(true)
    }

    fn run_listener(&self, shutdown: Arc<AtomicBool>, socket_path: PathBuf) {
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind socket: {}", e);
                return;
            }
        };

        self.set_permissions().ok();

        loop {
            if shutdown.load(Ordering::Relaxed) {
                println!("[IPC Listener] Shutdown received, stopping.");
                break;
            }

            match listener.accept() {
                Ok((stream, _addr)) => {
                    println!("accept a new client from {:#?}", _addr);
                    let _ = self.validate_connection(&stream).map(|_| {
                        let _ = self.handle_connection(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }

    fn read_request(
        &self,
        client_socket: &UnixStream,
    ) -> Result<Option<serde_json::Value>, IPCError> {
        let mut reader = BufReader::new(client_socket);
        let mut line = String::new();

        match reader.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let trimmed = line.trim_end_matches(|c: char| c.is_whitespace());
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    let obj: serde_json::Value = serde_json::from_str(trimmed)
                        .map_err(|e| IPCError::Other(format!("Failed to parse JSON: {}", e)))?;
                    Ok(Some(obj))
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => self.read_request(client_socket),
            Err(e) => Err(IPCError::Other(format!(
                "Failed to read from socket: {}",
                e
            ))),
        }
    }

    fn handle_connection(&self, client_socket: UnixStream) -> Result<(), IPCError> {
        if let Some(request) = self.read_request(&client_socket)? {
            self.handle_command(request)?;
            let response = self.create_success_response(None, "ok")?;
            self.send_response(client_socket, response)?;
        } else {
            self.clean_connection(client_socket)?;
        }
        Ok(())
    }

    fn handle_list_peers(&self, id: Option<u64>) {
        println!("handle_list_peers - ID: {:?}", id);
    }

    fn handle_send_file(
        &self,
        id: Option<u64>,
        path: String,
        peer: String,
        file_name: Option<String>,
    ) {
        println!(
            "handle_send_file - ID: {:?}, Path: {}, Peer: {}, File Name: {:?}",
            id, path, peer, file_name
        );
    }

    fn handle_get_status(&self, id: Option<u64>) {
        println!("handle_get_status - ID: {:?}", id);
    }

    fn handle_cancel_transfer(&self, id: Option<u64>, transfer_id: String) {
        println!(
            "handle_cancel_transfer - ID: {:?}, Transfer ID: {}",
            id, transfer_id
        );
    }

    fn handle_command(&self, raw: serde_json::Value) -> Result<(), IPCError> {
        match serde_json::from_value::<CommandRequest>(raw) {
            Ok(cmd) => match cmd {
                CommandRequest::ListPeers { id } => self.handle_list_peers(id),
                CommandRequest::SendFile {
                    id,
                    path,
                    peer,
                    file_name,
                } => self.handle_send_file(id, path, peer, file_name),
                CommandRequest::GetStatus { id } => self.handle_get_status(id),
                CommandRequest::CancelTransfer { id, transfer_id } => {
                    self.handle_cancel_transfer(id, transfer_id)
                }
            },
            Err(_) => return Err(IPCError::Other("Failed to parse command".to_string())),
        }

        Ok(())
    }

    fn create_success_response<T: Serialize>(
        &self,
        id: Option<u64>,
        data: T,
    ) -> Result<Vec<u8>, IPCError> {
        let response_id = id.unwrap_or(0);
        let response = SuccessMessage {
            id: response_id,
            status: "success".to_string(),
            data,
        };

        let json_string = serde_json::to_string(&response)
            .map_err(|e| IPCError::Other(format!("Failed to serialize response: {}", e)))?;

        let mut bytes = json_string.into_bytes();
        bytes.push(b'\n');

        Ok(bytes)
    }

    fn create_error_response(
        &self,
        id: Option<u64>,
        error: String,
        code: String,
    ) -> Result<Vec<u8>, IPCError> {
        let response_id = id.unwrap_or(0);
        let response = ErrorMessage {
            id: response_id,
            status: "error".to_string(),
            error,
            code,
        };

        let json_string = serde_json::to_string(&response)
            .map_err(|e| IPCError::Other(format!("Failed to serialize response: {}", e)))?;

        let mut bytes = json_string.into_bytes();
        bytes.push(b'\n');

        Ok(bytes)
    }

    fn send_response(&self, mut stream: UnixStream, response: Vec<u8>) -> Result<(), IPCError> {
        stream
            .write_all(&response)
            .map_err(|e| IPCError::Other(format!("Failed to write response: {}", e)))?;
        stream
            .flush()
            .map_err(|e| IPCError::Other(format!("Failed to flush response: {}", e)))?;
        Ok(())
    }

    fn clean_connection(&self, stream: UnixStream) -> Result<(), IPCError> {
        stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| IPCError::Other(format!("Failed to shutdown connection: {}", e)))?;
        Ok(())
    }

    pub fn shutdown(mut self) {
        if self.shutdown.load(Ordering::Relaxed) {
            if let Some(handle) = self.listener_handle.take() {
                let _ = handle.join().expect("Failed to join listener thread");
            }
        }
        println!("Shutdown complete.");
    }
}
