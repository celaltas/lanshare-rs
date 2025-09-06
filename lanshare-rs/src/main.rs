use std::process::exit;

fn main() {
    if let Err(e) = lanshare_core::run_server() {
        eprintln!("application error: {e}");
        exit(1);
    }
}
