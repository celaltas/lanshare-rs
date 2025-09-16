use std::net::{TcpListener, TcpStream};
use std::{io, thread};

use lanshare_proto::{FileMessage, MessageHeader};

use crate::storage::FileStorage;

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
    let header = match MessageHeader::read_from(&mut stream) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to read file header: {}", e);
            return;
        }
    };

    println!("Receiving file: {} ({} bytes)", header.name, header.size);

    let fs = match FileStorage::new("./storage") {
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("Failed to initialize file storage: {}", e);
            return;
        }
    };

    let mut tx = match fs.resume_transaction(&header.name) {
        Ok(tx) => {
            println!(
                "Resumed transaction: {} (already {} bytes written)",
                tx.id, tx.written_bytes
            );
            tx
        }
        Err(_) => match fs.create_transaction(&header.name, header.size, header.sha256) {
            Ok(tx) => {
                println!("Started new transaction: {}", tx.id);
                tx
            }
            Err(e) => {
                eprintln!("Failed to create transaction: {}", e);
                return;
            }
        },
    };

    let remaining = header.size.saturating_sub(tx.written_bytes);
    println!(
        "Expecting {} more bytes (total size = {}, written = {})",
        remaining, header.size, tx.written_bytes
    );

    let result = FileMessage::receive(&mut stream, &mut tx.writer(), header.size);

    match result {
        Ok(_) => match tx.commit() {
            Ok(_) => {
                println!("File saved successfully!");
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!(
                    "Upload incomplete for {}: {} of {} bytes written, waiting for resume",
                    header.name, tx.written_bytes, tx.total_size
                );
            }
            Err(e) => {
                eprintln!("Commit failed: {}", e);
                if let Err(rollback_err) = tx.rollback() {
                    eprintln!("Rollback also failed: {}", rollback_err);
                }
            }
        },
        Err(e) => {
            eprintln!("Transfer failed: {}", e);
            if let Err(rollback_err) = tx.rollback() {
                eprintln!("Rollback also failed: {}", rollback_err);
            } else {
                println!("Rollback done!");
            }
        }
    }
}
