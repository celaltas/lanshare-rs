use lanshare_app::use_cases::{receive_file::ReceiveFileUseCase, send_file::SendFileUseCase};
use lanshare_discovery::adapter::MdnsDiscoveryAdapter;
use lanshare_domain::{models::Peer, ports::DiscoveryPort};
use lanshare_network::adapter::TcpNetworkAdapter;
use lanshare_storage::adapter::LocalFileSystemAdapter;
use std::{process::exit, sync::Arc, thread};

fn main() {
    let storage_path = "./lanshare_storage";
    let storage_adapter = Arc::new(LocalFileSystemAdapter::new(storage_path).unwrap());
    let network_adapter = Arc::new(TcpNetworkAdapter::new());

    let receive_file_usecase = Arc::new(ReceiveFileUseCase::new(storage_adapter.clone()));
    let send_file_usecase = Arc::new(SendFileUseCase::new(
        storage_adapter.clone(),
        network_adapter.clone(),
    ));

    let discovery_adapter = Arc::new(MdnsDiscoveryAdapter::new().unwrap());

    // if let Err(e) = lanshare_core::run_server() {
    //     eprintln!("application error: {e}");
    //     exit(1);
    // }
    //
    let test_peer = Peer::new(
        "test peer".to_string(),
        "127.0.0.1:8080".parse().unwrap(),
        0,
    );

    if let Err(e) = discovery_adapter.broadcast_presence(&test_peer) {
        eprintln!("Failed to broadcast presence: {:?}", e);
    }

    let receive_uc_clone = receive_file_usecase.clone();
    thread::spawn(move || {
        if let Err(e) = TcpNetworkAdapter::start_listening(receive_uc_clone) {
            eprintln!("Network listener error: {:?}", e);
        }
    });

    println!("LanShare Daemon is running in the background.");
}
