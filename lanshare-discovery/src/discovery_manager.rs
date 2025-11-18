use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use crate::{events::PeerEvent, registry::Registry, service::DiscoveryService};

pub struct DiscoveryManager {
    service: DiscoveryService,
}

impl DiscoveryManager {
    pub fn new() -> Self {
        let registry = Registry::new();
        let shutdown = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::channel();
        let mut register_clone = registry.clone();
        let shutdown_clone = Arc::clone(&shutdown);
        let service = DiscoveryService::new(tx, shutdown_clone);
        let _listener_handle = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(e) => match e {
                        PeerEvent::PeerDiscovered(peer_info) => {
                            register_clone.add_peer(peer_info);
                        }
                        // PeerEvent::PeerUpdated(peer_info) => todo!(),
                        PeerEvent::PeerLost(name) => {
                            register_clone.remove_peer(&name);
                        }
                        PeerEvent::DiscoveryError(_) => todo!(),
                        _ => todo!(),
                    },
                    Err(e) => {
                        eprintln!("received error from discovery service: {}", e);
                        break;
                    }
                }
            }
        });

        let registry_cleaner = registry.clone();
        let shutdown_cleaner = Arc::clone(&shutdown);
        let _cleaner_handle = thread::spawn(move || {
            Self::cleaner(shutdown_cleaner, registry_cleaner);
        });

        Self { service }
    }

    fn cleaner(shutdown: Arc<AtomicBool>, mut registry: Registry) {
        println!("[Cleaner] Thread started, running every 30s");
        loop {
            if shutdown.load(Ordering::Relaxed) {
                println!("[Cleaner] Received shutdown signal, exiting gracefully.");
                break;
            }
            println!("[Cleaner] Cleaning up old entries...");
            registry.cleanup_stale(Duration::from_secs(30));
            println!("[Cleaner] Completed cycle.");

            if shutdown.load(Ordering::Relaxed) {
                println!("[Cleaner] Caught shutdown after action, exiting.");
                break;
            }

            thread::sleep(Duration::from_secs(30));
        }
        println!("[Cleaner] Thread stopped");
    }
}

impl DiscoveryApi for DiscoveryManager {
    fn list_peers(&self) -> Result<Vec<PeerApi>, ApiError> {
        let internal_peers = self.get_peers_internal();

        let api_peers: Vec<PeerApi> = internal_peers
            .into_iter()
            .map(|internal_peer| PeerApi {
                name: internal_peer.name,
                ip: internal_peer.ip.to_string(),
                port: internal_peer.port,
                last_seen: internal_peer.last_seen.to_rfc3339(),
            })
            .collect();

        Ok(api_peers)
    }
}
