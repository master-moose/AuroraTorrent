use anyhow::Result;
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

#[derive(Clone)]
struct AppState {
    torrents: Arc<Mutex<Vec<TorrentState>>>,
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting AuroraTorrent Engine...");

    let state = AppState {
        torrents: Arc::new(Mutex::new(Vec::new())),
    };

    // Initialize DHT
    let _dht = dht::init_dht().await?;

    // Start Streaming Server
    let stream_state = state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/stream/:id/:file_idx", get(stream_handler))
            .with_state(stream_state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
        info!("Streaming server listening on http://127.0.0.1:3000");
        axum::serve(listener, app).await.unwrap();
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
                .split('&')
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
            });
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
            }
            RpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(serde_json::json!({ "status": "started" })),
                error: None,
            }
        }
        RpcCommand::StreamTorrent { id } => {
            let mut torrents = state.torrents.lock().unwrap();
            if let Some(t) = torrents.iter_mut().find(|t| t.id == id) {
                t.status = "Streaming".to_string();
            }
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
        _ => RpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some("Method not implemented".into()),
        },
    }
}

async fn stream_handler(Path((id, file_idx)): Path<(String, usize)>, State(_state): State<AppState>) -> impl IntoResponse {
    // Mock streaming response
    format!("Streaming torrent {} file {}", id, file_idx)
}
