use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "params")]
pub enum RpcCommand {
    AddTorrent { magnet: String },
    ListTorrents,
    StartTorrent { id: String },
    PauseTorrent { id: String },
    RemoveTorrent { id: String },
    StreamTorrent { id: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorrentState {
    pub id: String,
    pub name: String,
    pub progress: f64,
    pub status: String, // "Downloading", "Seeding", "Paused"
    pub download_speed: u64,
    pub upload_speed: u64,
    pub total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(flatten)]
    pub command: RpcCommand,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<String>,
}

pub const PORT: u16 = 4000;
