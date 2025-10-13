use crate::events::PeerEvent;
use flume::RecvTimeoutError;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self};
use std::time::Duration;

const SERVICE_NAME: &str = "_lanshare._tcp.local.";

pub struct DiscoveryService {
    workers: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    event_sender: Sender<PeerEvent>,
    dns_daemon: ServiceDaemon,
    announcer_service: Option<ServiceInfo>,
}

impl DiscoveryService {
    pub fn new(event_sender: Sender<PeerEvent>, shutdown: Arc<AtomicBool>) -> Self {
        let mut workers = Vec::new();
        let dns_daemon = ServiceDaemon::new().expect("Failed to create daemon");
        let event_sender_clone = event_sender.clone();

        let own_id = "lanshare-rs".to_string();
        let own_id_clone = own_id.clone();
        let shutdown_clone = Arc::clone(&shutdown);
        let daemon_clone = dns_daemon.clone();
        let listener_handle = thread::spawn(move || {
            Self::listener(
                event_sender_clone,
                own_id_clone,
                shutdown_clone,
                daemon_clone,
            );
        });
        workers.push(listener_handle);

        let local_ip = Self::get_local_ip();
        println!("Detected local IP: {}", local_ip);

        let announcer_service = Self::announce_service(&dns_daemon, local_ip, own_id.clone());

        Self {
            workers,
            shutdown,
            event_sender,
            dns_daemon,
            announcer_service,
        }
    }

    fn worker<F>(
        name: String,
        action: F,
        shutdown: Arc<AtomicBool>,
        interval: Duration,
    ) -> thread::JoinHandle<()>
    where
        F: Fn() + Send + 'static,
    {
        thread::spawn(move || {
            println!("[{}] Thread started, running every {:?}", name, interval);
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    println!(
                        "Worker thread {} received shutdown signal, exiting gracefully.",
                        name
                    );
                    break;
                }

                action();
                println!("Worker thread {} completed cycle.", name);

                if shutdown.load(Ordering::Relaxed) {
                    println!(
                        "Worker thread {} caught shutdown after sleep, exiting.",
                        name
                    );
                    break;
                }
            }
            println!("[{}] Thread stopped", name);
        })
    }

    fn get_local_ip() -> String {
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                let target: SocketAddr = "8.8.8.8:80".parse().unwrap();
                if socket.connect(target).is_ok() {
                    if let Ok(addr) = socket.local_addr() {
                        return addr.ip().to_string();
                    }
                }
            }
            Err(_) => {}
        }
        eprintln!("Warning: Could not detect local IP, using fallback.");
        "127.0.0.1".to_string()
    }

    pub fn stop(mut self) {
        println!("Initiating graceful shutdown...");
        self.shutdown.store(true, Ordering::Relaxed);
        println!("Shutdown signal sent; workers will exit on next check.");

        if let Some(ref announcer_service) = self.announcer_service {
            let fullname = announcer_service.get_fullname();
            match self.dns_daemon.unregister(fullname) {
                Ok(rx) => {
                    if let Ok(status) = rx.recv_timeout(Duration::from_secs(5)) {
                        println!("[Shutdown] Unregister status: {:?}", status);
                    } else {
                        eprintln!("[Shutdown] Unregister timeout");
                    }
                }
                Err(e) => eprintln!("Unregister failed: {}", e),
            }
        }

        match self.dns_daemon.shutdown() {
            Ok(rx) => {
                if let Ok(status) = rx.recv_timeout(Duration::from_secs(5)) {
                    println!("[Shutdown] Daemon status: {:?}", status);
                } else {
                    eprintln!("[Shutdown] Daemon shutdown timeout");
                }
            }
            Err(e) => eprintln!("Daemon shutdown failed: {}", e),
        }

        for (i, worker) in self.workers.drain(..).enumerate() {
            match worker.join() {
                Ok(_) => println!("Worker {} joined successfully", i),
                Err(_) => eprintln!("Worker {} join timeout, detaching", i),
            }
        }
        println!("Shutdown complete.");
    }

    fn listener(
        event_sender: Sender<PeerEvent>,
        own_id: String,
        shutdown: Arc<AtomicBool>,
        daemon: ServiceDaemon,
    ) {
        println!("[Listener:{}] Thread started", own_id);
        let receiver = daemon
            .browse(SERVICE_NAME)
            .map_err(|e| {
                eprintln!("Failed to browse: {}", e);
                return;
            })
            .unwrap();

        let timeout = Duration::from_millis(100);
        let own_fullname = format!("{}.{}", own_id, SERVICE_NAME);

        loop {
            if shutdown.load(Ordering::Relaxed) {
                println!("[Listener:{}] Shutdown received, stopping.", own_id);
                let _ = daemon.stop_browse(SERVICE_NAME);
                break;
            }

            match receiver.recv_timeout(timeout) {
                Ok(event) => match event {
                    ServiceEvent::ServiceResolved(resolved) => {
                        let fullname = resolved.get_fullname().to_string();
                        if fullname == own_fullname {
                            println!("[Listener:{}] Ignoring own service: {}", own_id, fullname);
                            continue;
                        }
                        println!(
                            "[Listener:{}] Resolved peer: {} (port: {}, addrs: {:?})",
                            own_id,
                            fullname,
                            resolved.get_port(),
                            resolved.get_addresses()
                        );
                        if let Err(e) =
                            event_sender.send(PeerEvent::PeerDiscovered(resolved.into()))
                        {
                            eprintln!("Failed to send PeerDiscovered: {}", e);
                        }
                    }
                    ServiceEvent::ServiceRemoved(service_type, fullname) => {
                        if fullname == own_fullname {
                            continue;
                        }
                        println!(
                            "[Listener:{}] Peer removed: {} ({})",
                            own_id, fullname, service_type
                        );

                        let _ = event_sender.send(PeerEvent::PeerLost(fullname.to_string()));
                    }
                    other => {
                        println!("[Listener:{}] Other event: {:?}", own_id, other);
                    }
                },
                Err(e) => match e {
                    RecvTimeoutError::Timeout => {
                        continue;
                    }
                    _ => {
                        eprintln!("[Listener:{}] Recv error: {}, exiting.", own_id, e);
                        break;
                    }
                },
            }
        }
        println!("[Listener:{}] Thread stopped", own_id);
    }

    fn announce_service(daemon: &ServiceDaemon, ip: String, own_id: String) -> Option<ServiceInfo> {
        let instance_name = own_id;
        let host_name = format!("{}.local.", ip);
        let port: u16 = 5200;
        let properties = [("peer_id", "announcer"), ("version", "1.0")];

        let service = ServiceInfo::new(
            SERVICE_NAME,
            &instance_name,
            &host_name,
            ip.clone(),
            port,
            &properties[..],
        )
        .map_err(|e| {
            eprintln!("Failed to create service info: {}", e);
            e
        })
        .ok()?;

        println!("service info:{:?}", service);

        if let Err(e) = daemon.register(service.clone()) {
            eprintln!("Failed to register service: {}", e);
            return None;
        }

        println!("[Announce] Service registered: {}", service.get_fullname());
        Some(service)
    }
}
