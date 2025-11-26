use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use bridge::{RpcCommand, RpcRequest, RpcResponse, TorrentState, FileInfo, PeerInfo, TrackerInfo, PORT};
use librqbit::{Session, AddTorrentOptions, SessionOptions};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{info, error};

mod config;
use config::Config;

#[derive(Clone)]
struct AppState {
    session: Arc<Session>,
    config: Arc<Mutex<Config>>,
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    info!("Starting AuroraTorrent Engine with librqbit...");

    // Ensure download directory exists
    let config = Config::default();
    tokio::fs::create_dir_all(&config.download_path).await?;

    // Initialize librqbit session
    let session = Session::new(config.download_path.clone()).await?;
    let session = Arc::new(session);

    let state = AppState {
        session: session.clone(),
        config: Arc::new(Mutex::new(config)),
    };

    // Start Streaming Server (Placeholder for now, librqbit has its own stream handling usually, 
    // but we might need to proxy it or expose it differently. 
    // For now, we'll keep the structure but maybe point to librqbit's stream if possible)
    let stream_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/stream/:id/:file_idx", get(stream_handler))
            .with_state(stream_state);
            
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind streaming server: {}", e);
                return;
            }
        };
        
        info!("Streaming server listening on http://127.0.0.1:3000");
        if let Err(e) = axum::serve(listener, app).await {
            error!("Streaming server failed: {}", e);
        }
    });

    // Start RPC Server
    let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).await?;
    info!("RPC server listening on 127.0.0.1:{}", PORT);

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
                // info!("RPC Raw: {}", req_str);
                match serde_json::from_str::<RpcRequest>(&req_str) {
                    Ok(req) => {
                        let response = handle_rpc(req, &state).await;
                        let resp_bytes = serde_json::to_vec(&response).unwrap();
                        socket.write_all(&resp_bytes).await.ok();
                    }
                    Err(e) => {
                        error!("RPC Parse Error: {}", e);
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
            match state.session.add_torrent(&magnet, None).await {
                Ok(handle) => {
                    let id = handle.info_hash().to_hex();
                    RpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: Some(serde_json::json!({ "status": "added", "id": id })),
                        error: None,
                    }
                }
                Err(e) => RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(format!("Failed to add torrent: {}", e)),
                }
            }
        }
        RpcCommand::ListTorrents => {
            let handles = state.session.torrents();
            let mut torrents = Vec::new();
            
            for handle in handles {
                let info = handle.info();
                let stats = handle.stats();
                
                // Map librqbit state to our TorrentState
                // Note: This is a best-effort mapping.
                
                let files = info.files().iter().map(|f| FileInfo {
                    name: f.name.clone(),
                    size: f.len,
                    progress: 0.0, // TODO: Calculate per-file progress if possible
                }).collect();

                torrents.push(TorrentState {
                    id: handle.info_hash().to_hex(),
                    name: info.name.clone(),
                    progress: stats.progress, // Assuming 0.0 to 1.0
                    status: if stats.finished { "Seeding".into() } else { "Downloading".into() },
                    download_speed: stats.download_speed,
                    upload_speed: stats.upload_speed,
                    total_size: info.total_size,
                    files,
                    peers: vec![], // TODO: Populate peers
                    trackers: vec![], // TODO: Populate trackers
                });
            }

            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::to_value(&torrents).unwrap()),
                error: None,
            }
        }
        RpcCommand::StartTorrent { id } => {
            // librqbit starts automatically, but maybe we can pause/resume?
            // For now, just say started.
             RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ "status": "started" })),
                error: None,
            }
        }
        RpcCommand::StreamTorrent { id } => {
             RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ 
                    "status": "streaming",
                    "url": format!("http://127.0.0.1:3000/stream/{}/0", id)
                })),
                error: None,
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
            {
                let mut config = state.config.lock().unwrap();
                if let Some(p) = download_path { config.download_path = p; }
                // TODO: Apply speed limits to session
            }
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
    // TODO: Hook into librqbit's streaming capabilities
    // For now, return a placeholder
    axum::response::Response::builder()
        .header("Content-Type", "text/plain")
        .body(axum::body::Body::from(format!("Streaming for {} not yet implemented with librqbit", id)))
        .unwrap()
}
