use bincode::deserialize;
use database::{Database, SqlitePool};
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{RwLock, broadcast},
};
use types::{
    Block,
    config::Config,
    error::ChainError,
    p2p::{P2PConfig, P2PMessage, PeerConnection},
};

use std::{
    collections::HashMap,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct BlockChain {
    pub db: Database,
    peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
    message_tx: broadcast::Sender<P2PMessage>,
    node_id: String,
    config: P2PConfig,
    is_running: Arc<RwLock<bool>>,
}

impl Deref for BlockChain {
    type Target = Database;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for BlockChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl BlockChain {
    pub fn new(db: Database, config: Option<P2PConfig>) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            db,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_tx: tx,
            node_id: uuid::Uuid::new_v4().to_string(),
            config: config.unwrap_or_default(),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn from_config(config: Config) -> Self {
        let db = Database::new(
            SqlitePool::connect(&config.main_db).await.unwrap(),
            SqlitePool::connect(&config.private_db).await.unwrap(),
        );
        let config = config.peer_config.unwrap_or_default();
        Self::new(db, Some(config))
    }

    // Start P2P server
    pub async fn start_p2p_server(&self, bind_addr: SocketAddr) -> Result<(), ChainError> {
        let listener = TcpListener::bind(bind_addr).await?;
        *self.is_running.write().await = true;

        log::info!(
            "🚀 P2P server started on {} with id {}",
            bind_addr,
            self.node_id
        );

        // Start background tasks
        let blockchain = self.clone();
        tokio::spawn(async move {
            blockchain.peer_maintenance_loop().await;
        });

        // Accept incoming connections
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    if self.peers.read().await.len() >= self.config.max_peers {
                        log::warn!("Max peers reached, rejecting connection from {peer_addr}");
                        continue;
                    }

                    let mut blockchain = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = blockchain.handle_peer_connection(stream, peer_addr).await {
                            log::error!("Error handling peer {peer_addr}: {e}");
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {e}");
                }
            }
        }
    }

    // Connect to a peer
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<(), ChainError> {
        if self.peers.read().await.contains_key(&addr) {
            return Ok(()); // Already connected
        }
        log::info!("🔗 Connecting to peer: {addr}");

        let stream = tokio::time::timeout(self.config.connection_timeout, TcpStream::connect(addr))
            .await??;

        let mut blockchain = self.clone();

        if let Err(e) = blockchain.handle_peer_connection(stream, addr).await {
            log::error!("Error in outgoing connection to {addr}: {e}");
        }

        Ok(())
    }

    // Handle individual peer connection
    async fn handle_peer_connection(
        &mut self,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<(), ChainError> {
        log::debug!("New peer connection: {peer_addr}");

        // Add peer to peer list
        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_addr, PeerConnection::new(peer_addr));
        }

        // Perform handshake
        let chain_height = self
            .get_chain_height()
            .await
            .unwrap_or(0)
            .try_into()
            .unwrap_or_default();
        let hello_msg = P2PMessage::Hello {
            node_id: self.node_id.clone(),
            version: 1,
            chain_height,
        };

        self.send_message(&mut stream, &hello_msg).await?;

        // Handle messages
        let mut message_rx = self.message_tx.subscribe();

        loop {
            tokio::select! {
                // Handle incoming messages from peer
                result = self.read_message(&mut stream) => {
                    match result {
                        Ok(message) => {
                            // Extract stream handling into separate async block to avoid cycle
                            self.process_p2p_message(message, peer_addr, &mut stream).await?;
                        }
                        Err(_) => {
                            log::warn!("Connection closed by peer: {peer_addr}");
                            break;
                        }
                    }
                }

                // Forward broadcast messages to peer
                broadcast_result = message_rx.recv() => {
                    if let Ok(message) = broadcast_result {
                        // Don't send message back to sender
                        self.send_message(&mut stream, &message).await?;
                    }
                }
            }
        }

        // Clean up peer connection
        self.peers.write().await.remove(&peer_addr);
        log::warn!("Disconnected from peer: {peer_addr}");

        Ok(())
    }

    // Process P2P messages (separated to avoid async cycles)
    async fn process_p2p_message(
        &mut self,
        message: P2PMessage,
        peer_addr: SocketAddr,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        // Update last seen
        if let Some(peer) = self.peers.write().await.get_mut(&peer_addr) {
            peer.last_seen = Instant::now();
        }

        match message {
            P2PMessage::Hello {
                node_id,
                version: _,
                chain_height,
            } => {
                self.handle_hello_message(node_id, chain_height, peer_addr, stream)
                    .await
            }

            P2PMessage::HelloResponse {
                node_id,
                chain_height,
                accepted,
                ..
            } => {
                self.handle_hello_response(node_id, chain_height, accepted, peer_addr, stream)
                    .await
            }

            P2PMessage::BlockAnnouncement(block) => {
                log::info!(
                    "📢 Received block announcement: {} (height: {})",
                    block.hash,
                    block.height
                );
                self.handle_new_block(block).await?;
                Ok(())
            }

            P2PMessage::BlockRequest { hash } => self.handle_block_request(hash, stream).await,

            P2PMessage::ChainHeightRequest => {
                let height = self
                    .get_chain_height()
                    .await
                    .unwrap_or(0)
                    .try_into()
                    .unwrap();
                let response = P2PMessage::ChainHeightResponse { height };
                self.send_message(stream, &response).await
            }

            P2PMessage::GetBlocks {
                start_height,
                count,
            } => {
                self.handle_get_blocks_request(start_height, count, stream)
                    .await
            }

            P2PMessage::BlocksResponse { blocks } => self.handle_blocks_response(blocks).await,

            P2PMessage::GetPeers => self.handle_get_peers_request(stream).await,

            P2PMessage::PeersResponse { peers } => {
                self.handle_peers_response(peers, peer_addr).await
            }

            P2PMessage::Ping => {
                info!("🏓 Received ping from {peer_addr}");
                let response = P2PMessage::Pong;
                self.send_message(stream, &response).await
            }

            P2PMessage::Pong => {
                info!("🏓 Received pong from {peer_addr}");
                if let Some(peer) = self.peers.write().await.get_mut(&peer_addr) {
                    peer.last_ping = Some(Instant::now());
                }
                Ok(())
            }

            P2PMessage::Disconnect { reason } => {
                log::debug!("👋 Peer {peer_addr} disconnecting: {reason}");
                Err(ChainError::PeerError("Peer disconnected".into()))
            }

            _ => {
                log::debug!("❓ Unhandled message type from {peer_addr}");
                Ok(())
            }
        }
    }

    // Separate message handlers to reduce complexity
    async fn handle_hello_message(
        &self,
        node_id: String,
        chain_height: i64,
        peer_addr: SocketAddr,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        log::debug!("Received hello from {node_id} (height: {chain_height})");

        // Update peer info
        if let Some(peer) = self.peers.write().await.get_mut(&peer_addr) {
            peer.node_id = Some(node_id.clone());
            peer.chain_height = chain_height;
        }

        // Send response
        let our_height = self.get_chain_height().await?.try_into().unwrap();
        let response = P2PMessage::HelloResponse {
            node_id: self.node_id.clone(),
            version: 1,
            chain_height: our_height,
            accepted: true,
        };
        self.send_message(stream, &response).await?;

        dbg!(our_height, chain_height);

        // Start sync if peer has higher chain
        if chain_height > (our_height as i64) {
            self.request_chain_sync(peer_addr, stream).await?;
        }

        Ok(())
    }

    async fn handle_hello_response(
        &self,
        node_id: String,
        chain_height: i64,
        accepted: bool,
        peer_addr: SocketAddr,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        if accepted {
            log::debug!("Handshake accepted by {node_id} (height: {chain_height})");

            // Update peer info
            if let Some(peer) = self.peers.write().await.get_mut(&peer_addr) {
                peer.node_id = Some(node_id);
                peer.chain_height = chain_height;
            }

            // Start sync if peer has higher chain
            let our_height = self.get_chain_height().await.unwrap_or(0);
            if chain_height > our_height {
                self.request_chain_sync(peer_addr, stream).await?;
            }
        } else {
            log::debug!("Handshake rejected by peer {peer_addr}");
            return Err(ChainError::PeerError("Handshake rejected".into()));
        }

        Ok(())
    }

    async fn handle_block_request(
        &self,
        hash: String,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        if let Some(block) = self.get_block_by_hash(&hash).await? {
            let response = P2PMessage::BlockResponse { block: Some(block) };
            self.send_message(stream, &response).await
        } else {
            let response = P2PMessage::BlockResponse { block: None };
            self.send_message(stream, &response).await
        }
    }

    async fn handle_get_blocks_request(
        &self,
        start_height: i64,
        count: u32,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        let blocks = self.get_blocks_range(start_height, count).await?;
        let response = P2PMessage::BlocksResponse { blocks };
        self.send_message(stream, &response).await
    }

    async fn handle_blocks_response(&mut self, blocks: Vec<Block>) -> Result<(), ChainError> {
        log::debug!("Received {} blocks for sync", blocks.len());
        for block in blocks {
            let _ = self.add_block_to_chain(block).await; // TODO: handle multiple similar blocks
        }
        Ok(())
    }

    async fn handle_get_peers_request(&self, stream: &mut TcpStream) -> Result<(), ChainError> {
        let peers: Vec<SocketAddr> = self.peers.read().await.keys().cloned().collect();
        let response = P2PMessage::PeersResponse { peers };
        self.send_message(stream, &response).await
    }

    async fn handle_peers_response(
        &self,
        peers: Vec<SocketAddr>,
        peer_addr: SocketAddr,
    ) -> Result<(), ChainError> {
        log::info!("👥 Received {} peer addresses", peers.len());

        // Collect peers to connect to
        let mut peers_to_connect = Vec::new();
        {
            let current_peers = self.peers.read().await;
            for peer in peers {
                if !current_peers.contains_key(&peer) && peer != peer_addr {
                    peers_to_connect.push(peer);
                }
            }
        }

        // Connect to new peers outside of the async context
        for peer in peers_to_connect {
            self.spawn_peer_connection(peer);
        }

        Ok(())
    }

    // Helper function to spawn peer connections
    fn spawn_peer_connection(&self, peer_addr: SocketAddr) {
        let blockchain = self.clone();
        tokio::spawn(async move {
            let _ = blockchain.connect_to_peer(peer_addr).await;
        });
    }

    // Broadcast message to all peers
    pub async fn broadcast_message(&self, message: P2PMessage) -> Result<(), ChainError> {
        let _ = self.message_tx.send(message);
        Ok(())
    }

    // Announce new block to network
    pub async fn announce_block(&self, block: Block) -> Result<(), ChainError> {
        log::info!(
            "📢 Announcing new block: {} (height: {})",
            block.hash,
            block.height
        );
        self.broadcast_message(P2PMessage::BlockAnnouncement(block))
            .await
    }

    // Request chain synchronization
    async fn request_chain_sync(
        &self,
        peer_addr: SocketAddr,
        stream: &mut TcpStream,
    ) -> Result<(), ChainError> {
        let our_height = self.get_chain_height().await.unwrap_or(0);

        if let Some(peer) = self.peers.write().await.get_mut(&peer_addr) {
            peer.is_syncing = true;
        }

        log::info!("🔄 Starting chain sync from height {our_height} with {peer_addr}");

        let sync_msg = P2PMessage::GetBlocks {
            start_height: our_height.try_into().unwrap(),
            count: self.config.sync_batch_size,
        };

        self.send_message(stream, &sync_msg).await
    }

    // Peer maintenance (ping, cleanup)
    async fn peer_maintenance_loop(&self) {
        let mut interval = tokio::time::interval(self.config.ping_interval);

        loop {
            interval.tick().await;

            let peers: Vec<SocketAddr> = self.peers.read().await.keys().cloned().collect();

            log::info!("🔍 Peer maintenance: {} peers connected", peers.len());

            for peer_addr in peers {
                // Check if peer is still alive
                let should_disconnect = {
                    if let Some(peer) = self.peers.read().await.get(&peer_addr) {
                        peer.last_seen.elapsed() > self.config.ping_interval * 3
                    } else {
                        false
                    }
                };

                if should_disconnect {
                    log::debug!("Peer {peer_addr} timed out, removing");
                    self.peers.write().await.remove(&peer_addr);
                }
            }

            // Broadcast ping to check peer health
            let _ = self.broadcast_message(P2PMessage::Ping).await;
        }
    }

    // Message serialization
    async fn send_message(
        &self,
        stream: &mut TcpStream,
        message: &P2PMessage,
    ) -> Result<(), ChainError> {
        let data = bincode::serialize(message)?;
        let len = data.len() as u32;

        // Send length prefix
        stream.write_all(&len.to_be_bytes()).await?;
        // Send message data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn read_message(&self, stream: &mut TcpStream) -> Result<P2PMessage, ChainError> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Check message size limit
        if len > self.config.max_message_size {
            return Err(ChainError::Other("Message too large".into()));
        }

        // Read message data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;

        let message: P2PMessage = deserialize(&data)?;
        Ok(message)
    }

    // Blockchain operations (these would interact with your Database)
    async fn get_chain_height(&self) -> Result<i64, ChainError> {
        let height = self.db.get_height().await?;
        Ok(height)
    }

    async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>, ChainError> {
        let res = self.db.get_block_by_hash(hash).await?;
        Ok(Some(res))
    }

    #[allow(unused)]
    async fn get_blocks_range(
        &self,
        start_height: i64,
        count: u32,
    ) -> Result<Vec<Block>, ChainError> {
        Ok(self
            .db
            .get_blocks_in_range(start_height, count as i64)
            .await?)
    }

    async fn handle_new_block(&mut self, block: Block) -> Result<i64, ChainError> {
        log::debug!("Processing new block: {}", block.hash);
        self.add_block_to_chain(block).await
    }

    async fn add_block_to_chain(&mut self, block: Block) -> Result<i64, ChainError> {
        // self.db
        //     .add_public_key(
        //         block.creator_pub_key.as_bytes(),
        //         &block.creator,
        //         &block.signature_pub_key_hash,
        //         block.height as i32,
        //     )
        //     .await;
        let res = self.db.add_block(&block).await?;
        Ok(res)
    }

    // Get connected peers info
    pub async fn get_peers_info(&self) -> Vec<PeerConnection> {
        self.peers.read().await.values().cloned().collect()
    }

    // Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    // Stop P2P networking
    pub async fn stop(&self) {
        *self.is_running.write().await = false;
        log::info!("🛑 P2P networking stopped");
    }
}

pub async fn start_node(
    blockchain: BlockChain,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{port}").parse()?;
    blockchain.start_p2p_server(addr).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_blockchain_creation() {
        let db = Database::new_in_memory();
        let blockchain = BlockChain::new(db, None);

        assert_eq!(blockchain.peer_count().await, 0);
        assert!(!blockchain.node_id.is_empty());
    }
}
