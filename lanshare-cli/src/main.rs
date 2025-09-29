use lanshare_proto::FileMessage;
use std::net::TcpStream;

pub enum Command {
    Send,
    SendPartial,
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: lanshare-cli <command> [options]");
        eprintln!("Commands:");
        eprintln!("  send <file_path> <ip_address>  Send a file to the specified IP address");
        eprintln!("  send_partiak <file_path> <ip_address> <cutoff>  Send a file in partially to the specified IP address");
        std::process::exit(1);
    }
    let command = match args[1].as_str() {
        "send" => Command::Send,
        "send_partial" => Command::SendPartial,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    };
    match command {
        Command::Send => {
            if args.len() != 4 {
                eprintln!("Usage: lanshare-cli send <file_path> <ip_address>");
                std::process::exit(1);
            }
            let file_path = &args[2];
            let ip_address = &args[3];
            let path = std::path::Path::new(file_path);
            if !path.exists() || !path.is_file() {
                eprintln!("File not found: {}", file_path);
                std::process::exit(1);
            }
            match TcpStream::connect(format!("{}:8080", ip_address)) {
                Ok(mut stream) => {
                    if let Err(e) = FileMessage::send(&mut stream, path) {
                        eprintln!("Failed to send file: {}", e);
                        std::process::exit(1);
                    } else {
                        println!("File sent successfully to {}", ip_address);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", ip_address, e);
                    std::process::exit(1);
                }
            }
        }
        Command::SendPartial => {
            if args.len() != 5 {
                eprintln!("Usage: lanshare-cli send_partial <file_path> <ip_address> cutoff");
                std::process::exit(1);
            }
            let file_path = &args[2];
            let ip_address = &args[3];
            let cutoff = match args[4].parse::<u64>() {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("Cutoff must be a valid number");
                    std::process::exit(1);
                }
            };
            let path = std::path::Path::new(file_path);
            if !path.exists() || !path.is_file() {
                eprintln!("File not found: {}", file_path);
                std::process::exit(1);
            }
            match TcpStream::connect(format!("{}:8080", ip_address)) {
                Ok(mut stream) => {
                    if let Err(e) = FileMessage::send_partial(&mut stream, path, cutoff) {
                        eprintln!("Failed to send file: {}", e);
                        std::process::exit(1);
                    } else {
                        println!(
                            "File partially sent ({} bytes) successfully to {}",
                            cutoff, ip_address
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", ip_address, e);
                    std::process::exit(1);
                }
            }
        }
    }
}
