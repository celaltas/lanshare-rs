use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::{io, thread};

use lanshare_discovery::DiscoveryManager;
use lanshare_proto::{FileMessage, MessageHeader};

use crate::storage::FileStorage;
use crate::transaction::Transaction;

pub fn run_server() -> io::Result<()> {
    let _manager = DiscoveryManager::new();
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running at 8080");

    let fs = FileStorage::new("./storage")?;
    let fs_arc = Arc::new(fs);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let fs_clone = Arc::clone(&fs_arc);
                thread::spawn(|| handle_connection(s, fs_clone));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, fs: Arc<FileStorage>) {
    let header = match MessageHeader::read_from(&mut stream) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to read file header: {}", e);
            return;
        }
    };

    println!("Receiving file: {} ({} bytes)", header.name, header.size);

    let mut tx = match fs.get_or_create_transaction(&header) {
        Ok(tx) => tx,
        Err(_) => return,
    };

    let result = process_file_transfer(stream, &mut tx, header.size);

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

fn process_file_transfer(
    mut stream: TcpStream,
    tx: &mut Transaction,
    header_size: u64,
) -> io::Result<()> {
    let already_written = tx.written_bytes;
    let mut writer = tx.writer();
    let remaining = header_size.saturating_sub(already_written);

    if remaining > 0 {
        println!(
            "Need to skip {} bytes and read {} more bytes",
            already_written, remaining
        );
        if already_written > 0 {
            println!(
                "Skipping {} bytes that we already wrote...",
                already_written
            );
            FileMessage::skip_bytes(&mut stream, already_written)
        } else {
            println!("Reading remaining {} bytes...", remaining);
            FileMessage::receive(&mut stream, &mut writer, remaining)
        }
    } else {
        println!("File already complete, skipping entire stream");
        FileMessage::skip_bytes(&mut stream, header_size)
    }
}
