use std::net::{TcpListener, TcpStream};
use std::{io, thread};

use lanshare_proto::read_file_message;

pub fn run_server() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running at 8080");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| handle_connection(s));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    match read_file_message(&mut stream) {
        Ok(file_message) => {
            println!(
                "Received file: {} ({} bytes)",
                file_message.header.name, file_message.header.size
            );
            // TODO: Here you can save the file content to disk or process it as needed
        }
        Err(e) => {
            eprintln!("Failed to read file message: {}", e);
        }
    }
}
