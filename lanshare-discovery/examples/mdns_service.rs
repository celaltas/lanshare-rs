// test_peer.rs
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::time::Duration;

const SERVICE_NAME: &'static str = "_lanshare._tcp.local.";

fn main() {
    let mdns = ServiceDaemon::new().unwrap();

    let my_service = ServiceInfo::new(
        SERVICE_NAME,
        "peer-different",
        "test.local.",
        "192.168.1.100",
        8080,
        &[("id", "DIFFERENT_PEER")][..],
    )
    .unwrap();

    mdns.register(my_service).unwrap();
    println!("Peer registered, waiting...");

    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
