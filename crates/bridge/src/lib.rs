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
    GetConfig,
    SetConfig { 
        download_path: Option<String>,
        max_download_speed: Option<u64>,
        max_upload_speed: Option<u64>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub progress: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerInfo {
    pub ip: String,
    pub client: String,
    pub down_speed: u64,
    pub up_speed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackerInfo {
    pub url: String,
    pub status: String,
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
    pub files: Vec<FileInfo>,
    pub peers: Vec<PeerInfo>,
    pub trackers: Vec<TrackerInfo>,
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
