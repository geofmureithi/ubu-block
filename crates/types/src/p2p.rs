use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use crate::Block;
// P2P Message Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    // Handshake
    Hello {
        node_id: String,
        version: u32,
        chain_height: i64,
    },
    HelloResponse {
        node_id: String,
        version: u32,
        chain_height: i64,
        accepted: bool,
    },

    // Block synchronization
    BlockAnnouncement(Block),
    BlockRequest {
        hash: String,
    },
    BlockResponse {
        block: Option<Block>,
    },

    // Chain synchronization
    ChainHeightRequest,
    ChainHeightResponse {
        height: i64,
    },
    GetBlocks {
        start_height: i64,
        count: u32,
    },
    BlocksResponse {
        blocks: Vec<Block>,
    },

    // Peer discovery
    GetPeers,
    PeersResponse {
        peers: Vec<SocketAddr>,
    },

    // Keep-alive
    Ping,
    Pong,

    // Disconnect
    Disconnect {
        reason: String,
    },
}

// Peer connection state
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub addr: SocketAddr,
    pub node_id: Option<String>,
    pub last_seen: Instant,
    pub chain_height: i64,
    pub is_syncing: bool,
    pub connection_time: Instant,
    pub bytes_sent: i64,
    pub bytes_received: i64,
    pub last_ping: Option<Instant>,
}

impl PeerConnection {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            node_id: None,
            last_seen: Instant::now(),
            chain_height: 0,
            is_syncing: false,
            connection_time: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            last_ping: None,
        }
    }
}

// P2P Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub max_peers: usize,
    pub ping_interval: Duration,
    pub connection_timeout: Duration,
    pub sync_batch_size: u32,
    pub max_message_size: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            ping_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            sync_batch_size: 100,
            max_message_size: 10 * 1024 * 1024, // 10MB
        }
    }
}
