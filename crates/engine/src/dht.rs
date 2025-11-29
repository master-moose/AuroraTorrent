use anyhow::Result;
use mainline::Dht;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};

/// DHT manager for peer discovery
pub struct DhtManager {
    dht: Dht,
}

impl DhtManager {
    /// Initialize the DHT
    pub fn new() -> Result<Self> {
        let dht = Dht::client()?;
        info!("DHT initialized");
        Ok(Self { dht })
    }

    /// Search for peers with a given info hash
    pub fn get_peers(&self, info_hash: [u8; 20]) -> Vec<SocketAddr> {
        let id = match mainline::Id::from_bytes(info_hash) {
            Ok(id) => id,
            Err(e) => {
                warn!("Invalid info hash for DHT: {}", e);
                return Vec::new();
            }
        };

        let mut peers = Vec::new();

        // get_peers returns an iterator, collect results
        let iterator = self.dht.get_peers(id);
        for peer_addrs in iterator {
            for addr in peer_addrs {
                peers.push(SocketAddr::V4(addr));
            }
            // Limit to first 50 peers for efficiency
            if peers.len() >= 50 {
                break;
            }
        }

        if !peers.is_empty() {
            info!(
                "DHT found {} peers for {}",
                peers.len(),
                hex::encode(info_hash)
            );
        }
        peers
    }

    /// Announce ourselves to the DHT
    pub fn announce(&self, info_hash: [u8; 20], port: u16) -> Result<()> {
        let id = mainline::Id::from_bytes(info_hash)?;
        self.dht.announce_peer(id, Some(port))?;
        info!(
            "Announced to DHT: {} on port {}",
            hex::encode(info_hash),
            port
        );
        Ok(())
    }
}

/// Initialize and return a shared DHT manager
pub async fn init_dht() -> Result<Arc<DhtManager>> {
    Ok(Arc::new(DhtManager::new()?))
}
