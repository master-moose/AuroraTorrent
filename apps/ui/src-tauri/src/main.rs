// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use engine;
use bridge::PORT;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use tokio::time::{timeout, Duration};

#[tauri::command]
async fn rpc_request(request: String) -> Result<String, String> {
    let timeout_duration = Duration::from_secs(5);

    // Connect to the engine via TCP with timeout
    let mut stream = timeout(timeout_duration, TcpStream::connect(format!("127.0.0.1:{}", PORT)))
        .await
        .map_err(|_| "Connection timed out".to_string())?
        .map_err(|e| e.to_string())?;

    // Write request with timeout
    timeout(timeout_duration, stream.write_all(request.as_bytes()))
        .await
        .map_err(|_| "Write timed out".to_string())?
        .map_err(|e| e.to_string())?;

    // Signal EOF to server so it processes and then closes the connection
    stream.shutdown().await.map_err(|e| e.to_string())?;

    // Read response until EOF with timeout
    let mut buf = Vec::new();
    timeout(timeout_duration, stream.read_to_end(&mut buf))
        .await
        .map_err(|_| "Read timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let response = String::from_utf8_lossy(&buf).to_string();
    Ok(response)
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
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
