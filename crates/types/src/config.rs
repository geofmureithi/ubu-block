use crate::p2p::P2PConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct Config {
    pub mode: Option<Mode>,
    pub main_db: String,
    pub private_db: String,
    pub peer_config: Option<P2PConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Mode {
    Observer { peer_addr: String },
}
