// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bridge::{
    AddTorrentParams, Category, FilePriority, RssDownloadRule, RssFeed,
    SessionStats, TorrentLimits,
};
use engine::config::Config;
use engine::Engine;
// serde used by bridge types
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Shared engine state accessible from all Tauri commands
type SharedEngine = Arc<Mutex<Option<Engine>>>;

// =============================================================================
// TORRENT MANAGEMENT
// =============================================================================

#[tauri::command]
async fn list_torrents(
    engine: State<'_, SharedEngine>,
) -> Result<Vec<bridge::TorrentState>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.list_torrents().await)
}

#[tauri::command]
async fn add_torrent(
    engine: State<'_, SharedEngine>,
    magnet: String,
    params: Option<AddTorrentParams>,
) -> Result<serde_json::Value, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_magnet(&magnet, params).await;
    if result.success {
        Ok(result.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn add_torrent_file(
    engine: State<'_, SharedEngine>,
    name: String,
    content: String,
    params: Option<AddTorrentParams>,
) -> Result<serde_json::Value, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_torrent_file(&name, &content, params).await;
    if result.success {
        Ok(result.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn start_torrent(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.start_torrent(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn pause_torrent(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.pause_torrent(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn stop_torrent(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.stop_torrent(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn remove_torrent(
    engine: State<'_, SharedEngine>,
    id: String,
    delete_files: Option<bool>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine
        .remove_torrent(&id, delete_files.unwrap_or(false))
        .await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn force_recheck(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.force_recheck(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn force_reannounce(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.force_reannounce(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn rename_torrent(
    engine: State<'_, SharedEngine>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.rename_torrent(&id, &new_name).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn set_torrent_category(
    engine: State<'_, SharedEngine>,
    id: String,
    category: Option<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.set_torrent_category(&id, category).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn add_torrent_tags(
    engine: State<'_, SharedEngine>,
    id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_torrent_tags(&id, tags).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn remove_torrent_tags(
    engine: State<'_, SharedEngine>,
    id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.remove_torrent_tags(&id, tags).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn set_torrent_limits(
    engine: State<'_, SharedEngine>,
    id: String,
    limits: TorrentLimits,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.set_torrent_limits(&id, limits).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn toggle_sequential_download(
    engine: State<'_, SharedEngine>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.toggle_sequential(&id, enabled).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn toggle_first_last_piece_priority(
    engine: State<'_, SharedEngine>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.toggle_first_last_piece(&id, enabled).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// STREAMING
// =============================================================================

#[tauri::command]
async fn stream_torrent(
    engine: State<'_, SharedEngine>,
    id: String,
) -> Result<serde_json::Value, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.stream_torrent(&id).await;
    if result.success {
        Ok(result.data.unwrap_or(serde_json::json!({})))
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// TRACKERS
// =============================================================================

#[tauri::command]
async fn add_trackers(
    engine: State<'_, SharedEngine>,
    id: String,
    trackers: Vec<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_trackers(&id, trackers).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn remove_trackers(
    engine: State<'_, SharedEngine>,
    id: String,
    trackers: Vec<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.remove_trackers(&id, trackers).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// FILE PRIORITY
// =============================================================================

#[tauri::command]
async fn get_torrent_files(
    engine: State<'_, SharedEngine>,
    id: String,
) -> Result<Vec<bridge::FileInfo>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.get_torrent_files(&id).await;
    if result.success {
        Ok(result.data.unwrap_or_default())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn set_file_priority(
    engine: State<'_, SharedEngine>,
    torrent_id: String,
    file_index: usize,
    priority: FilePriority,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine
        .set_file_priority(&torrent_id, file_index, priority)
        .await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn set_files_priority(
    engine: State<'_, SharedEngine>,
    torrent_id: String,
    file_indices: Vec<usize>,
    priority: FilePriority,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine
        .set_files_priority(&torrent_id, file_indices, priority)
        .await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// QUEUE MANAGEMENT
// =============================================================================

#[tauri::command]
async fn queue_move_up(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.queue_move_up(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn queue_move_down(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.queue_move_down(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn queue_move_top(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.queue_move_top(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn queue_move_bottom(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.queue_move_bottom(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// CONFIGURATION
// =============================================================================

#[tauri::command]
async fn get_config(engine: State<'_, SharedEngine>) -> Result<Config, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_config().await)
}

#[tauri::command]
async fn set_config(engine: State<'_, SharedEngine>, config: Config) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.set_full_config(config).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn toggle_alt_speed(engine: State<'_, SharedEngine>, enabled: bool) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.toggle_alt_speed(enabled).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// CATEGORIES AND TAGS
// =============================================================================

#[tauri::command]
async fn create_category(
    engine: State<'_, SharedEngine>,
    name: String,
    save_path: Option<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.create_category(name, save_path).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn edit_category(
    engine: State<'_, SharedEngine>,
    name: String,
    save_path: Option<String>,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.edit_category(name, save_path).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn delete_category(engine: State<'_, SharedEngine>, name: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.delete_category(&name).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn get_categories(engine: State<'_, SharedEngine>) -> Result<HashMap<String, Category>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_categories().await)
}

#[tauri::command]
async fn create_tag(engine: State<'_, SharedEngine>, tag: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.create_tag(tag).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn delete_tag(engine: State<'_, SharedEngine>, tag: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.delete_tag(&tag).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn get_tags(engine: State<'_, SharedEngine>) -> Result<Vec<String>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_tags().await)
}

// =============================================================================
// RSS FEEDS
// =============================================================================

#[tauri::command]
async fn get_rss_feeds(engine: State<'_, SharedEngine>) -> Result<Vec<RssFeed>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_rss_feeds().await)
}

#[tauri::command]
async fn add_rss_feed(
    engine: State<'_, SharedEngine>,
    url: String,
    name: Option<String>,
) -> Result<RssFeed, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_rss_feed(url, name).await;
    if result.success {
        Ok(result.data.unwrap())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn remove_rss_feed(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.remove_rss_feed(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn refresh_rss_feed(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.refresh_rss_feed(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn get_rss_articles(
    engine: State<'_, SharedEngine>,
    feed_id: Option<String>,
) -> Result<Vec<bridge::RssArticle>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_rss_articles(feed_id).await)
}

#[tauri::command]
async fn get_rss_rules(engine: State<'_, SharedEngine>) -> Result<Vec<RssDownloadRule>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_rss_rules().await)
}

#[tauri::command]
async fn add_rss_rule(
    engine: State<'_, SharedEngine>,
    rule: RssDownloadRule,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.add_rss_rule(rule).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn remove_rss_rule(engine: State<'_, SharedEngine>, id: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.remove_rss_rule(&id).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// SEARCH
// =============================================================================

#[tauri::command]
async fn search_torrents(
    engine: State<'_, SharedEngine>,
    query: String,
    plugins: Option<Vec<String>>,
    category: Option<String>,
) -> Result<Vec<bridge::SearchResult>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.search_torrents(&query, plugins, category).await;
    if result.success {
        Ok(result.data.unwrap_or_default())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[tauri::command]
async fn get_search_plugins(engine: State<'_, SharedEngine>) -> Result<Vec<bridge::SearchPlugin>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_search_plugins().await)
}

#[tauri::command]
async fn enable_search_plugin(
    engine: State<'_, SharedEngine>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.enable_search_plugin(&name, enabled).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// TORRENT CREATION
// =============================================================================

#[tauri::command]
async fn create_torrent(
    engine: State<'_, SharedEngine>,
    source_path: String,
    trackers: Vec<String>,
    comment: Option<String>,
    is_private: bool,
    piece_size: Option<u64>,
) -> Result<Vec<u8>, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine
        .create_torrent(&source_path, trackers, comment, is_private, piece_size)
        .await;
    if result.success {
        Ok(result.data.unwrap_or_default())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// =============================================================================
// SESSION STATS
// =============================================================================

#[tauri::command]
async fn get_session_stats(engine: State<'_, SharedEngine>) -> Result<SessionStats, String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    Ok(engine.get_session_stats().await)
}

// =============================================================================
// UTILITIES
// =============================================================================

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn ban_peer(engine: State<'_, SharedEngine>, ip: String) -> Result<(), String> {
    let guard = engine.lock().await;
    let engine = guard.as_ref().ok_or("Engine not initialized")?;
    let result = engine.ban_peer(&ip).await;
    if result.success {
        Ok(())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

fn main() {
    let engine_state: SharedEngine = Arc::new(Mutex::new(None));
    let engine_state_clone = engine_state.clone();

    tauri::Builder::default()
        .manage(engine_state)
        .setup(move |app| {
            // Initialize the engine
            let engine_state = engine_state_clone.clone();
            let _app_handle = app.handle();
            
            tauri::async_runtime::spawn(async move {
                match Engine::new().await {
                    Ok(engine) => {
                        *engine_state.lock().await = Some(engine);
                        println!("Engine initialized successfully");
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize engine: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Torrent management
            list_torrents,
            add_torrent,
            add_torrent_file,
            start_torrent,
            pause_torrent,
            stop_torrent,
            remove_torrent,
            force_recheck,
            force_reannounce,
            rename_torrent,
            set_torrent_category,
            add_torrent_tags,
            remove_torrent_tags,
            set_torrent_limits,
            toggle_sequential_download,
            toggle_first_last_piece_priority,
            // Streaming
            stream_torrent,
            // Trackers
            add_trackers,
            remove_trackers,
            // File priority
            get_torrent_files,
            set_file_priority,
            set_files_priority,
            // Queue
            queue_move_up,
            queue_move_down,
            queue_move_top,
            queue_move_bottom,
            // Config
            get_config,
            set_config,
            toggle_alt_speed,
            // Categories and tags
            create_category,
            edit_category,
            delete_category,
            get_categories,
            create_tag,
            delete_tag,
            get_tags,
            // RSS
            get_rss_feeds,
            add_rss_feed,
            remove_rss_feed,
            refresh_rss_feed,
            get_rss_articles,
            get_rss_rules,
            add_rss_rule,
            remove_rss_rule,
            // Search
            search_torrents,
            get_search_plugins,
            enable_search_plugin,
            // Torrent creation
            create_torrent,
            // Session stats
            get_session_stats,
            // Utilities
            open_folder,
            ban_peer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
