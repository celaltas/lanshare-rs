use std::{sync::mpsc, thread};

use lanshare_discovery::{DiscoveryService, Registry};

pub struct DiscoveryManager {
    service: DiscoveryService,
}

impl DiscoveryManager {
    pub fn new() -> Self {
        let registry = Registry::new();
        let (tx, rx) = mpsc::channel();
        let service = DiscoveryService::new(registry, tx);
        let _listener_handle = thread::spawn(move || {
            println!("manager listener");
            loop {
                match rx.recv() {
                    Ok(_) => {
                        println!("new event published");
                    }
                    Err(_) => {
                        println!("Discovery service stopped");
                        break;
                    }
                }
            }
        });

        Self { service }
    }
}
