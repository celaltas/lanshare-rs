use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::peer::PeerInfo;

#[derive(Debug, Clone)]
pub struct Registry {
    data: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

impl Registry {
    pub fn new() -> Self {
        let data = Arc::new(RwLock::new(HashMap::new()));
        Self { data }
    }

    pub fn add_peer(&mut self, peer: PeerInfo) -> bool {
        if let Ok(mut guard) = self.data.write() {
            let _ = guard.insert(peer.id.clone(), peer);
            true
        } else {
            false
        }
    }
    pub fn remove_peer(&mut self, id: &str) -> Option<PeerInfo> {
        if let Ok(mut guard) = self.data.write() {
            guard.remove(id)
        } else {
            None
        }
    }
    pub fn get_peer(&self, id: &str) -> Option<PeerInfo> {
        if let Ok(guard) = self.data.read() {
            guard.get(id).cloned()
        } else {
            None
        }
    }
    pub fn list_peers(&self) -> Vec<PeerInfo> {
        if let Ok(guard) = self.data.read() {
            guard.values().map(|p| p.clone()).collect()
        } else {
            Vec::new()
        }
    }
    pub fn cleanup_stale(&mut self, timeout: Duration) -> usize {
        let mut remove_ids = Vec::new();
        if let Ok(guard) = self.data.read() {
            let ids = guard
                .iter()
                .filter_map(|(id, peer_info)| {
                    if peer_info.is_stale(timeout) {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            remove_ids.extend(ids);
        }

        if let Ok(mut guard) = self.data.write() {
            remove_ids.into_iter().map(|id| guard.remove(&id)).count()
        } else {
            0
        }
    }
}
