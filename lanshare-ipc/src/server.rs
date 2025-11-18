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
        let shutdown = self.shutdown.clone();
        let handle = thread::spawn(move || self_clone.run_listener(shutdown));
        self.listener_handle = Some(handle);
        Ok(())
    }

    fn initialize_socket(&self) -> Result<(), IPCError> {
        let is_file_exist = fs::metadata(self.socket_path.as_path()).is_ok();
        if is_file_exist {
            fs::remove_file(self.socket_path.as_path())
                .map_err(|_e| IPCError::Other("Failed to remove socket".into()))?;
        }
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

    fn run_listener(&self, shutdown: Arc<AtomicBool>) {
        let listener = match self.bind_socket() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind socket: {:?}", e);
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
                    match self.validate_connection(&stream) {
                        Ok(_) => match self.handle_connection(stream) {
                            Ok(_) => println!("Connection handled successfully"),
                            Err(_) => eprintln!("Error handling connection"),
                        },
                        Err(e) => {
                            eprintln!("Error validating connection: {:?}", e);
                        }
                    };
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
        let res = match reader.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(n) => {
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
        };

        res
    }

    fn handle_connection(&self, client_socket: UnixStream) -> Result<(), IPCError> {
        match self.read_request(&client_socket) {
            Ok(Some(request)) => match self.handle_command(request) {
                Ok(response) => self.send_response(client_socket, response),
                Err(handle_error) => {
                    let response = self.create_error_response(
                        None,
                        "Failed to handle command".to_string(),
                        "HANDLE_ERROR".to_string(),
                    )?;
                    let _ = self.send_response(client_socket, response);
                    Err(handle_error)
                }
            },
            Ok(None) => self.clean_connection(client_socket),
            Err(read_error) => {
                let response = self.create_error_response(
                    None,
                    "Failed to read request".to_string(),
                    "READ_ERROR".to_string(),
                )?;
                let _ = self.send_response(client_socket, response);
                Err(read_error)
            }
        }
    }

    fn handle_list_peers(&self, id: Option<u64>) -> Result<Vec<u8>, IPCError> {
        let peers = vec!["test1", "test2"];
        Ok(self.create_success_response(id, peers)?)
    }

    fn handle_send_file(
        &self,
        id: Option<u64>,
        path: String,
        peer: String,
        file_name: Option<String>,
    ) -> Result<Vec<u8>, IPCError> {
        Ok(self.create_success_response(id, "ok")?)
    }

    fn handle_get_status(&self, id: Option<u64>) -> Result<Vec<u8>, IPCError> {
        Ok(self.create_success_response(id, "ok")?)
    }

    fn handle_cancel_transfer(
        &self,
        id: Option<u64>,
        transfer_id: String,
    ) -> Result<Vec<u8>, IPCError> {
        Ok(self.create_success_response(id, "ok")?)
    }

    fn handle_command(&self, raw: serde_json::Value) -> Result<Vec<u8>, IPCError> {
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

#[cfg(test)]
mod tests {
    use std::{io::Read, time::Duration};

    use super::*;

    #[test]
    fn test_handle_list_peers() {
        let socket_path = PathBuf::from("/tmp/lanshare-ipc-test.sock");
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut server = IPCServer::new(socket_path.clone(), shutdown);
        let server_handle = thread::spawn(move || server.start().unwrap());
        thread::sleep(Duration::from_millis(50));

        let mut stream = UnixStream::connect(socket_path).unwrap();
        stream.write_all(b"hello world\n").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert_eq!(response, "ok");
        server_handle.join().expect("Server thread panicked");

        println!("Test completed successfully!");
    }
}
