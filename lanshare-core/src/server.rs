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

    let mut tx = match fs.create_transaction(&header.name) {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to create transaction: {}", e);
            return;
        }
    };

    let result = FileMessage::receive(&mut stream, tx.writer(), header.size);

    match result {
        Ok(_) => {
            if let Err(e) = tx.commit() {
                eprintln!("Commit failed: {}", e);
                if let Err(rollback_err) = tx.rollback() {
                    eprintln!("Rollback also failed: {}", rollback_err);
                }
            } else {
                println!("File saved successfully!");
            }
        }
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
