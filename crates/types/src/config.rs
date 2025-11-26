use std::collections::HashMap;

use crate::p2p::P2PConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct Config {
    pub mode: Option<Mode>,
    pub main_db: String,
    pub private_db: String,
    pub peer_config: Option<P2PConfig>,
    pub peers: Option<HashMap<String, String>>,
    pub http_addr: Option<String>,
    pub node_addr: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Mode {
    Observer { peer_addr: String },
}
