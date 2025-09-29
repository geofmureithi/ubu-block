use tokio::time::error::Elapsed;

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("Timeout error: {0}")]
    TimeoutError(#[from] Elapsed),
    #[error("Peer error: {0}")]
    PeerError(String),
    #[error("Other error: {0}")]
    Other(String),
}
