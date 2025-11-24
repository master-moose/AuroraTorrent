use anyhow::Result;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use bridge::{RpcCommand, RpcRequest, RpcResponse, TorrentState, PORT};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

mod dht;
mod storage;
mod config;

use config::Config;
use std::fs;

#[derive(Clone)]
struct AppState {
    torrents: Arc<Mutex<Vec<TorrentState>>>,
    config: Arc<Mutex<Config>>,
    has_ffmpeg: bool,
}

const STATE_FILE: &str = "torrents.json";

#[derive(Serialize, Deserialize)]
struct PersistedState {
    torrents: Vec<TorrentState>,
    config: Config,
}

fn save_state(state: &AppState) {
    let torrents = state.torrents.lock().unwrap().clone();
    let config = state.config.lock().unwrap().clone();
    let data = PersistedState { torrents, config };
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(STATE_FILE, json);
    }
}

fn load_state() -> (Vec<TorrentState>, Config) {
    if let Ok(content) = fs::read_to_string(STATE_FILE) {
        if let Ok(data) = serde_json::from_str::<PersistedState>(&content) {
            return (data.torrents, data.config);
        }
    }
    (Vec::new(), Config::default())
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    info!("Starting AuroraTorrent Engine...");

    // Check for ffmpeg at startup
    let has_ffmpeg = tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .is_ok();
    
    if has_ffmpeg {
        info!("FFmpeg detected. Transcoding enabled.");
    } else {
        info!("FFmpeg not found. Transcoding disabled.");
    }

    let (loaded_torrents, loaded_config) = load_state();
    info!("Loaded {} torrents from state.", loaded_torrents.len());

    let state = AppState {
        torrents: Arc::new(Mutex::new(loaded_torrents)),
        config: Arc::new(Mutex::new(loaded_config)),
        has_ffmpeg,
    };

    // Initialize DHT
    let _dht = dht::init_dht().await?;

    // Start Streaming Server
    let stream_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/stream/:id/:file_idx", get(stream_handler))
            .with_state(stream_state);
            
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind streaming server on 127.0.0.1:3000: {}", e);
                return;
            }
        };
        
        info!("Streaming server listening on http://127.0.0.1:3000");
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Streaming server failed: {}", e);
        }
    });

    // Start RPC Server
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?;
    info!("RPC server listening on 127.0.0.1:{}", PORT);

    // Simulation Loop (Temporary)
    let sim_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let mut torrents = sim_state.torrents.lock().unwrap();
            for t in torrents.iter_mut() {
                if t.status == "Downloading" || t.status == "Streaming" {
                    if t.progress < 1.0 {
                        t.progress += 0.01; // +1% per second
                        t.download_speed = 1024 * 1024 * 2; // 2 MB/s mock speed
                    } else {
                        t.status = "Seeding".to_string();
                        t.progress = 1.0;
                        t.download_speed = 0;
                    }
                }
            }
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let mut buf = [0; 4096];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => return,
                };

                let req_str = String::from_utf8_lossy(&buf[..n]);
                info!("RPC Raw: {}", req_str);
                match serde_json::from_str::<RpcRequest>(&req_str) {
                    Ok(req) => {
                        let response = handle_rpc(req, &state).await;
                        let resp_bytes = serde_json::to_vec(&response).unwrap();
                        socket.write_all(&resp_bytes).await.ok();
                    }
                    Err(e) => {
                        info!("RPC Parse Error: {}", e);
                    }
                }
            }
        });
    }
}

async fn handle_rpc(req: RpcRequest, state: &AppState) -> RpcResponse<serde_json::Value> {
    info!("Received command: {:?}", req.command);
    match req.command {
        RpcCommand::AddTorrent { magnet } => {
            let mut torrents = state.torrents.lock().unwrap();
            let id = format!("{:x}", md5::compute(&magnet)); // Simple ID gen
            
            // Parse magnet link for display name (dn)
            let name = magnet
                .split(|c| c == '&' || c == '?')
                .find(|part| part.starts_with("dn="))
                .map(|part| part.trim_start_matches("dn=").to_string())
                .and_then(|s| urlencoding::decode(&s).ok().map(|s| s.into_owned()))
                .unwrap_or_else(|| "Unknown Torrent".to_string());

            torrents.push(TorrentState {
                id: id.clone(),
                name,
                progress: 0.0,
                status: "Downloading".to_string(),
                download_speed: 0,
                upload_speed: 0,
                total_size: 1024 * 1024 * 100, // Mock size
                files: vec![
                    bridge::FileInfo { name: "movie.mkv".to_string(), size: 1024 * 1024 * 100, progress: 0.0 },
                    bridge::FileInfo { name: "sample.txt".to_string(), size: 1024, progress: 0.0 },
                ],
                peers: vec![
                    bridge::PeerInfo { ip: "192.168.1.5".to_string(), client: "Transmission".to_string(), down_speed: 1024 * 500, up_speed: 0 },
                ],
                trackers: vec![
                    bridge::TrackerInfo { url: "udp://tracker.opentrackr.org:1337".to_string(), status: "Working".to_string() },
                ],
            });
            save_state(state);
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ "status": "added", "id": id })),
                error: None,
            }
        }
        RpcCommand::ListTorrents => {
            let torrents = state.torrents.lock().unwrap();
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::to_value(&*torrents).unwrap()),
                error: None,
            }
        }
        RpcCommand::StartTorrent { id } => {
            let mut torrents = state.torrents.lock().unwrap();
            if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
                t.status = "Downloading".to_string();
                save_state(state);
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
        RpcCommand::StreamTorrent { id } => {
            let mut torrents = state.torrents.lock().unwrap();
            if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
                t.status = "Streaming".to_string();
                RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::json!({ 
                        "status": "streaming",
                        "url": format!("http://127.0.0.1:3000/stream/{}/0", id)
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
            let config = state.config.lock().unwrap();
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::to_value(&*config).unwrap()),
                error: None,
            }
        }
        RpcCommand::SetConfig { download_path, max_download_speed, max_upload_speed } => {
            let mut config = state.config.lock().unwrap();
            if let Some(p) = download_path { config.download_path = p; }
            if let Some(s) = max_download_speed { config.max_download_speed = s; }
            if let Some(s) = max_upload_speed { config.max_upload_speed = s; }
            
            save_state(state);
            
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ "status": "updated" })),
                error: None,
            }
        }
        _ => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some("Method not implemented".into()),
        },
    }
}

async fn stream_handler(Path((id, file_idx)): Path<(String, usize)>, State(state): State<AppState>) -> impl IntoResponse {
    // In a real implementation, we would fetch the file path from 'state' using 'id' and 'file_idx'.
    // For now, we'll mock the file path and type.
    let file_name = "movie.mkv"; // Mock file name
    let _file_path = format!("/tmp/{}", file_name);

    let is_native = file_name.ends_with(".mp4") || file_name.ends_with(".webm") || file_name.ends_with(".ogg");
    
    if !is_native && state.has_ffmpeg {
        info!("Transcoding {}...", file_name);
        // Spawn ffmpeg to transcode to fragmented MP4
        // Note: This is a skeleton. In real code, we'd pipe the stdout to the response body.
        // Since we don't have real file I/O yet, we return a message indicating intent.
        return axum::response::Response::builder()
            .header("Content-Type", "video/mp4")
            .body(axum::body::Body::from("Transcoding stream placeholder..."))
            .unwrap();
    }

    // Fallback or Native
    info!("Streaming {} directly...", file_name);
    axum::response::Response::builder()
        .header("Content-Type", "text/plain")
        .body(axum::body::Body::from(format!("Streaming torrent {} file {} directly", id, file_idx)))
        .unwrap()
}
