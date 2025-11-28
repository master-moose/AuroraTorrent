//! AuroraTorrent Engine
//!
//! Core BitTorrent engine with streaming support

use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bridge::{FileInfo, PeerInfo, RpcCommand, RpcRequest, RpcResponse, TrackerInfo, TorrentState, PORT};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use session::{SessionState, TorrentSession};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub mod bencode;
mod config;
mod dht;
pub mod peer;
pub mod piece;
pub mod session;
pub mod storage;
pub mod torrent;
pub mod tracker;

use config::Config;
use torrent::{MagnetInfo, TorrentMetainfo};

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Active torrent sessions by ID
    sessions: Arc<DashMap<String, Arc<TorrentSession>>>,
    /// Torrent metadata cache (for magnets that need metadata)
    metainfo_cache: Arc<DashMap<String, TorrentMetainfo>>,
    /// Configuration
    config: Arc<RwLock<Config>>,
    /// Our peer ID
    peer_id: [u8; 20],
    /// Has FFmpeg for transcoding
    has_ffmpeg: bool,
    /// Listen port for incoming peers
    listen_port: u16,
    /// Next listen port (incremented per torrent to avoid conflicts)
    next_port: Arc<std::sync::atomic::AtomicU16>,
}

const STATE_FILE: &str = "aurora_state.json";

#[derive(Serialize, Deserialize)]
struct PersistedState {
    torrents: Vec<PersistedTorrent>,
    config: Config,
}

#[derive(Serialize, Deserialize)]
struct PersistedTorrent {
    id: String,
    name: String,
    magnet: Option<String>,
    torrent_file: Option<Vec<u8>>,
    progress: f64,
    status: String,
    total_size: u64,
}

impl AppState {
    async fn save_state(&self) -> Result<()> {
        let config = self.config.read().await.clone();
        let mut torrents = Vec::new();

        for entry in self.sessions.iter() {
            let session = entry.value();
            let stats = session.stats().await;
            let state = session.state().await;

            torrents.push(PersistedTorrent {
                id: entry.key().clone(),
                name: session.metainfo.name.clone(),
                magnet: None, // TODO: Store magnet or torrent file for resume
                torrent_file: None,
                progress: stats.progress,
                status: match state {
                    SessionState::Downloading => "Downloading",
                    SessionState::Seeding => "Seeding",
                    SessionState::Paused => "Paused",
                    _ => "Stopped",
                }
                .to_string(),
                total_size: session.metainfo.total_size,
            });
        }

        let data = PersistedState { torrents, config };
        let json = serde_json::to_string_pretty(&data)?;

        let state_path = self.config.read().await.download_path.clone();
        let state_file = PathBuf::from(&state_path).join(STATE_FILE);

        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let temp_file = state_file.with_extension("tmp");
        let mut file = fs::File::create(&temp_file).await?;
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;
        fs::rename(&temp_file, &state_file).await?;

        Ok(())
    }

    async fn load_state(&self) -> Result<()> {
        let state_path = self.config.read().await.download_path.clone();
        let state_file = PathBuf::from(&state_path).join(STATE_FILE);

        if !state_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&state_file).await?;
        let data: PersistedState = serde_json::from_str(&content)?;

        // Apply config
        *self.config.write().await = data.config;

        // TODO: Resume torrents from stored magnet/torrent files
        info!("Loaded {} torrents from state", data.torrents.len());

        Ok(())
    }

    async fn torrent_state(&self, id: &str, session: &TorrentSession) -> TorrentState {
        let stats = session.stats().await;
        let state = session.state().await;
        let peer_stats = session.peer_stats().await;

        // Get piece manager progress info
        let file_progresses: Vec<f64> = {
            let pieces = session.piece_manager();
            let pm = pieces.read().await;
            let total_pieces = pm.total_pieces();
            let bitfield: Vec<bool> = (0..total_pieces).map(|i| pm.have[i]).collect();
            drop(pm);
            
            session
                .metainfo
                .files
                .iter()
                .map(|f| {
                    if stats.progress >= 1.0 {
                        1.0
                    } else {
                        let start_piece = (f.offset / session.metainfo.piece_length) as usize;
                        let end_piece =
                            ((f.offset + f.length - 1) / session.metainfo.piece_length) as usize;
                        let total = end_piece - start_piece + 1;
                        let complete = (start_piece..=end_piece)
                            .filter(|&p| p < total_pieces && bitfield[p])
                            .count();
                        complete as f64 / total as f64
                    }
                })
                .collect()
        };

        TorrentState {
            id: id.to_string(),
            name: session.metainfo.name.clone(),
            progress: stats.progress,
            status: match state {
                SessionState::Starting => "Starting".to_string(),
                SessionState::Downloading => "Downloading".to_string(),
                SessionState::Seeding => "Seeding".to_string(),
                SessionState::Paused => "Paused".to_string(),
                SessionState::Stopped => "Stopped".to_string(),
                SessionState::Error => "Error".to_string(),
            },
            download_speed: stats.download_rate,
            upload_speed: stats.upload_rate,
            total_size: session.metainfo.total_size,
            files: session
                .metainfo
                .files
                .iter()
                .zip(file_progresses)
                .map(|(f, progress)| FileInfo {
                    name: f.path.to_string_lossy().to_string(),
                    size: f.length,
                    progress,
                })
                .collect(),
            peers: peer_stats
                .iter()
                .map(|p| PeerInfo {
                    ip: p.addr.to_string(),
                    client: p.client.clone(),
                    down_speed: p.download_rate,
                    up_speed: p.upload_rate,
                })
                .collect(),
            trackers: session
                .metainfo
                .announce_list
                .iter()
                .flatten()
                .chain(session.metainfo.announce.iter())
                .map(|url| TrackerInfo {
                    url: url.clone(),
                    status: "Active".to_string(),
                })
                .collect(),
        }
    }
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    info!("Starting AuroraTorrent Engine v0.1.0");

    // Check for ffmpeg
    let has_ffmpeg = tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .is_ok();

    if has_ffmpeg {
        info!("FFmpeg detected - transcoding enabled");
    } else {
        info!("FFmpeg not found - transcoding disabled");
    }

    let config = Config::default();
    let download_path = config.download_path.clone();

    // Ensure download directory exists
    fs::create_dir_all(&download_path).await?;

    let state = AppState {
        sessions: Arc::new(DashMap::new()),
        metainfo_cache: Arc::new(DashMap::new()),
        config: Arc::new(RwLock::new(config)),
        peer_id: peer::generate_peer_id(),
        has_ffmpeg,
        listen_port: 6881,
        next_port: Arc::new(std::sync::atomic::AtomicU16::new(6882)),
    };

    // Load previous state
    if let Err(e) = state.load_state().await {
        warn!("Failed to load state: {}", e);
    }

    // Initialize DHT
    let _dht = dht::init_dht().await?;

    // Start streaming server
    let stream_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/stream/:id/:file_idx", get(stream_handler))
            .route("/stream/:id", get(stream_default_handler))
            .with_state(stream_state);

        let listener = match TcpListener::bind("127.0.0.1:3000").await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind streaming server: {}", e);
                return;
            }
        };

        info!("Streaming server on http://127.0.0.1:3000");
        if let Err(e) = axum::serve(listener, app).await {
            error!("Streaming server error: {}", e);
        }
    });

    // Start RPC server
    let rpc_listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?;
    info!("RPC server on 127.0.0.1:{}", PORT);

    // Auto-save state periodically
    let save_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = save_state.save_state().await {
                warn!("Failed to save state: {}", e);
            }
        }
    });

    loop {
        let (mut socket, addr) = rpc_listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(_) => return,
                };

                let req_str = String::from_utf8_lossy(&buf[..n]);
                match serde_json::from_str::<RpcRequest>(&req_str) {
                    Ok(req) => {
                        let response = handle_rpc(req, &state).await;
                        let resp_bytes = serde_json::to_vec(&response).unwrap();
                        if socket.write_all(&resp_bytes).await.is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        warn!("RPC parse error from {}: {}", addr, e);
                    }
                }
            }
        });
    }
}

async fn handle_rpc(req: RpcRequest, state: &AppState) -> RpcResponse<serde_json::Value> {
    match req.command {
        RpcCommand::AddTorrent { magnet } => {
            match TorrentMetainfo::from_magnet(&magnet) {
                Ok(magnet_info) => {
                    let id = magnet_info.info_hash_hex();

                    if state.sessions.contains_key(&id) {
                        return RpcResponse {
                            jsonrpc: "2.0".into(),
                            id: req.id,
                            result: None,
                            error: Some("Torrent already exists".into()),
                        };
                    }

                    // For magnets, we need to get metadata from peers
                    // For now, create a placeholder - full metadata exchange TBD
                    info!("Added magnet: {} ({})", magnet_info.name, id);

                    // Create a mock session for now until we implement metadata exchange
                    // In a full implementation, we'd use DHT/peers to get the info dict

                    RpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: Some(serde_json::json!({
                            "status": "added",
                            "id": id,
                            "name": magnet_info.name
                        })),
                        error: None,
                    }
                }
                Err(e) => RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Invalid magnet: {}", e)),
                },
            }
        }

        RpcCommand::AddTorrentFile { name: _, content } => {
            // Decode base64 content
            let bytes = match base64_decode(&content) {
                Ok(b) => b,
                Err(e) => {
                    return RpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: None,
                        error: Some(format!("Invalid torrent file: {}", e)),
                    }
                }
            };

            match TorrentMetainfo::from_bytes(&bytes) {
                Ok(metainfo) => {
                    let id = metainfo.info_hash_hex();

                    if state.sessions.contains_key(&id) {
                        return RpcResponse {
                            jsonrpc: "2.0".into(),
                            id: req.id,
                            result: None,
                            error: Some("Torrent already exists".into()),
                        };
                    }

                    let download_path = PathBuf::from(&state.config.read().await.download_path);
                    let port = state.next_port.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let session = Arc::new(TorrentSession::new(
                        metainfo.clone(),
                        download_path,
                        port,
                    ));

                    // Start the session
                    session.clone().start().await;

                    state.sessions.insert(id.clone(), session);
                    state.metainfo_cache.insert(id.clone(), metainfo.clone());

                    info!("Added torrent: {} ({})", metainfo.name, id);

                    RpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: Some(serde_json::json!({
                            "status": "added",
                            "id": id,
                            "name": metainfo.name
                        })),
                        error: None,
                    }
                }
                Err(e) => RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Failed to parse torrent: {}", e)),
                },
            }
        }

        RpcCommand::ListTorrents => {
            let mut torrents = Vec::new();
            for entry in state.sessions.iter() {
                let t = state.torrent_state(entry.key(), entry.value()).await;
                torrents.push(t);
            }

            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::to_value(&torrents).unwrap()),
                error: None,
            }
        }

        RpcCommand::StartTorrent { id } => {
            if let Some(session) = state.sessions.get(&id) {
                session.resume().await;
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::json!({ "status": "started" })),
                    error: None,
                }
            } else {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Torrent {} not found", id)),
                }
            }
        }

        RpcCommand::PauseTorrent { id } => {
            if let Some(session) = state.sessions.get(&id) {
                session.pause().await;
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::json!({ "status": "paused" })),
                    error: None,
                }
            } else {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Torrent {} not found", id)),
                }
            }
        }

        RpcCommand::RemoveTorrent { id } => {
            if let Some((_, session)) = state.sessions.remove(&id) {
                session.stop().await;
                state.metainfo_cache.remove(&id);

                if let Err(e) = state.save_state().await {
                    warn!("Failed to save state after removal: {}", e);
                }

                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::json!({ "status": "removed" })),
                    error: None,
                }
            } else {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Torrent {} not found", id)),
                }
            }
        }

        RpcCommand::StreamTorrent { id } => {
            if let Some(session) = state.sessions.get(&id) {
                // Enable sequential downloading for streaming
                session.set_sequential(true).await;

                // Prioritize first pieces for quick start
                let pieces_to_prioritize: Vec<u32> = (0..10).collect();
                session.prioritize_pieces(pieces_to_prioritize).await;

                // Find first streamable file
                let file_idx = session
                    .metainfo
                    .files
                    .iter()
                    .position(|f| {
                        let name = f.path.to_string_lossy().to_lowercase();
                        name.ends_with(".mp4")
                            || name.ends_with(".mkv")
                            || name.ends_with(".webm")
                            || name.ends_with(".mp3")
                            || name.ends_with(".flac")
                            || name.ends_with(".ogg")
                            || name.ends_with(".m4a")
                            || name.ends_with(".avi")
                    })
                    .unwrap_or(0);

                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::json!({
                        "status": "streaming",
                        "url": format!("http://127.0.0.1:3000/stream/{}/{}", id, file_idx)
                    })),
                    error: None,
                }
            } else {
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Torrent {} not found", id)),
                }
            }
        }

        RpcCommand::GetConfig => {
            let config = state.config.read().await.clone();
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::to_value(&config).unwrap()),
                error: None,
            }
        }

        RpcCommand::SetConfig {
            download_path,
            max_download_speed,
            max_upload_speed,
        } => {
            {
                let mut config = state.config.write().await;
                if let Some(p) = download_path {
                    config.download_path = p;
                }
                if let Some(s) = max_download_speed {
                    config.max_download_speed = s;
                }
                if let Some(s) = max_upload_speed {
                    config.max_upload_speed = s;
                }
            }

            if let Err(e) = state.save_state().await {
                warn!("Failed to save config: {}", e);
            }

            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ "status": "updated" })),
                error: None,
            }
        }
    }
}

#[derive(Deserialize)]
struct StreamQuery {
    range: Option<String>,
}

async fn stream_handler(
    Path((id, file_idx)): Path<(String, usize)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Torrent not found"))
                .unwrap()
        }
    };

    let file = match session.metainfo.files.get(file_idx) {
        Some(f) => f,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .unwrap()
        }
    };

    let file_size = file.length;
    let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();
    let content_type = mime_guess::from_path(&file.path)
        .first_or_octet_stream()
        .to_string();

    // Parse Range header
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| parse_range(s, file_size));

    let (start, end) = range.unwrap_or((0, file_size - 1));
    let content_length = end - start + 1;

    let data_start = file.offset + start;
    let _data_end = file.offset + end;

    // Check if data is available
    let pieces = session.piece_manager();
    {
        let pm = pieces.read().await;
        
        // For streaming, we need to wait for pieces if not available
        if !pm.is_range_available(data_start, content_length) {
            // Prioritize needed pieces
            let needed = pm.pieces_for_range(data_start, content_length);
            drop(pm);
            session.prioritize_pieces(needed).await;

            // Return "please wait" or partial content
            return Response::builder()
                .status(StatusCode::ACCEPTED)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("Buffering..."))
                .unwrap();
        }
    }

    // Read data from files
    let read_result = {
        let pm = pieces.read().await;
        pm.read_data(data_start, content_length as usize).await
    };

    match read_result {
        Ok(data) => {
            let status = if range.is_some() {
                StatusCode::PARTIAL_CONTENT
            } else {
                StatusCode::OK
            };

            let mut response = Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, content_length)
                .header(header::ACCEPT_RANGES, "bytes");

            if range.is_some() {
                response = response.header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                );
            }

            response.body(Body::from(data)).unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Read error: {}", e)))
            .unwrap(),
    }
}

async fn stream_default_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Stream first file by default
    stream_handler(Path((id, 0)), State(state), headers).await
}

fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let start: u64 = if parts[0].is_empty() {
        // Suffix range: -500 means last 500 bytes
        let suffix: u64 = parts[1].parse().ok()?;
        file_size.saturating_sub(suffix)
    } else {
        parts[0].parse().ok()?
    };

    let end: u64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    if start > end || start >= file_size {
        return None;
    }

    Some((start, std::cmp::min(end, file_size - 1)))
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Simple base64 decode (for data URLs, strip prefix)
    let data = if let Some(idx) = input.find(",") {
        &input[idx + 1..]
    } else {
        input
    };

    // Decode base64
    let mut result = Vec::new();
    let chars: Vec<u8> = data.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();

    const DECODE_TABLE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, 0, -1, -1, -1, 0, 1, 2, 3, 4, 5,
        6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1,
        -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
        47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    for chunk in chars.chunks(4) {
        let mut buffer = 0u32;
        let mut valid_bytes = 0;

        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                break;
            }
            if c >= 128 {
                return Err("Invalid base64 character".to_string());
            }
            let val = DECODE_TABLE[c as usize];
            if val < 0 {
                return Err("Invalid base64 character".to_string());
            }
            buffer = (buffer << 6) | (val as u32);
            valid_bytes = i + 1;
        }

        match valid_bytes {
            4 => {
                result.push((buffer >> 16) as u8);
                result.push((buffer >> 8) as u8);
                result.push(buffer as u8);
            }
            3 => {
                buffer <<= 6;
                result.push((buffer >> 16) as u8);
                result.push((buffer >> 8) as u8);
            }
            2 => {
                buffer <<= 12;
                result.push((buffer >> 16) as u8);
            }
            _ => {}
        }
    }

    Ok(result)
}
