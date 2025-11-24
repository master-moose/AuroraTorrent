// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use engine;
use bridge::{RpcRequest, RpcResponse, PORT};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tauri::command]
async fn rpc_request(request: String) -> Result<String, String> {
    // Connect to the engine via TCP
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", PORT))
        .await
        .map_err(|e| e.to_string())?;

    stream.write_all(request.as_bytes()).await.map_err(|e| e.to_string())?;

    let mut buf = [0; 4096];
    let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;

    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    Ok(response)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Spawn the engine in a separate thread
            tauri::async_runtime::spawn(async {
                if let Err(e) = engine::run().await {
                    eprintln!("Engine error: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![rpc_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
