use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::exit;

const SOCKET_PATH: &str = "/tmp/lanshare.sock";

#[derive(Debug)]
enum Command {
    List,
    Send { file_path: String, peer: String },
}

impl Command {
    fn from_args(args: &[String]) -> Result<Self, CliError> {
        let verb = args.get(1).ok_or(CliError::NoCommand)?;

        match verb.as_str() {
            "list" => Ok(Command::List),
            "send" => {
                let file_path = args
                    .get(2)
                    .ok_or(CliError::MissingArgument("file_path"))?
                    .clone();
                let peer = args
                    .get(3)
                    .ok_or(CliError::MissingArgument("peer"))?
                    .clone();
                Ok(Command::Send { file_path, peer })
            }
            unknown => Err(CliError::UnknownCommand(unknown.to_string())),
        }
    }

    fn to_request(&self) -> serde_json::Value {
        match self {
            Command::List => serde_json::json!({
                "command": "list_peers",
                "id": 1
            }),
            Command::Send { file_path, peer } => serde_json::json!({
                "command": "send_file",
                "id": 2,
                "path": file_path,
                "peer": peer
            }),
        }
    }
}

#[derive(Debug)]
enum CliError {
    NoCommand,
    UnknownCommand(String),
    MissingArgument(&'static str),
    DaemonConnect(std::io::Error),
    DaemonWrite(std::io::Error),
    DaemonRead(std::io::Error),
    DaemonHungUp,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::NoCommand => write!(f, "No command provided."),
            CliError::UnknownCommand(cmd) => write!(f, "Unknown command: '{}'", cmd),
            CliError::MissingArgument(arg) => write!(f, "Missing required argument: <{}>", arg),
            CliError::DaemonConnect(e) => write!(
                f,
                "Could not connect to LanShare daemon. Is it running?\n  ({})",
                e
            ),
            CliError::DaemonWrite(e) => write!(f, "Failed to send request to daemon: {}", e),
            CliError::DaemonRead(e) => write!(f, "Failed to read response from daemon: {}", e),
            CliError::DaemonHungUp => write!(f, "Daemon closed the connection unexpectedly."),
        }
    }
}

struct DaemonClient {
    stream: UnixStream,
}

impl DaemonClient {
    fn connect() -> Result<Self, CliError> {
        let stream = UnixStream::connect(SOCKET_PATH).map_err(CliError::DaemonConnect)?;
        Ok(Self { stream })
    }

    fn send_request(&mut self, payload: &serde_json::Value) -> Result<(), CliError> {
        let mut line = payload.to_string();
        line.push('\n');
        self.stream
            .write_all(line.as_bytes())
            .map_err(CliError::DaemonWrite)
    }

    fn read_response(&mut self) -> Result<String, CliError> {
        let mut reader = BufReader::new(&self.stream);
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(0) => Err(CliError::DaemonHungUp),
            Ok(_) => Ok(buf),
            Err(e) => Err(CliError::DaemonRead(e)),
        }
    }

    fn round_trip(&mut self, payload: &serde_json::Value) -> Result<String, CliError> {
        self.send_request(payload)?;
        self.read_response()
    }
}

fn print_response(raw: &str) {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(parsed) => println!("{}", serde_json::to_string_pretty(&parsed).unwrap()),
        Err(_) => println!("Response: {}", raw.trim()),
    }
}

fn print_usage() {
    eprintln!("Usage: lanshare-cli <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  list                         List peers on the network");
    eprintln!("  send <file_path> <peer>      Send a file to the specified peer");
}

fn print_error(err: &CliError) {
    eprintln!("Error: {}", err);
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().collect();

    let command = Command::from_args(&args)?;
    let request = command.to_request();

    let mut client = DaemonClient::connect()?;
    let response = client.round_trip(&request)?;

    print_response(&response);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        print_error(&e);

        if matches!(
            e,
            CliError::NoCommand | CliError::UnknownCommand(_) | CliError::MissingArgument(_)
        ) {
            eprintln!();
            print_usage();
        }

        exit(1);
    }
}
