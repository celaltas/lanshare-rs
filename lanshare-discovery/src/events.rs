use crate::peer::PeerInfo;

pub enum PeerEvent {
    PeerDiscovered(PeerInfo),
    PeerUpdated(PeerInfo),
    PeerLost(String),
    ServiceStarted,
    ServiceStopped,
    DiscoveryError(String),
}
