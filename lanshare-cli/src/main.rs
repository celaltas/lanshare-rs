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
        eprintln!(
            "  send_partiak <file_path> <ip_address> <cutoff>  Send a file in partially to the specified IP address"
        );
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
        Command::Send => todo!(),
        Command::SendPartial => todo!(),
    }
}
