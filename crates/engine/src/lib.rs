//! AuroraTorrent Engine
//!
//! Core BitTorrent engine with streaming support, RSS, search, and full qBittorrent feature parity

use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bridge::{
    AddTorrentParams, Category, FileInfo, FilePriority, PeerInfo, RssArticle, RssDownloadRule,
    RssFeed, SearchPlugin, SearchResult, SessionStats, TorrentLimits, TorrentState, TrackerInfo,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use session::{SessionState, TorrentSession};
// sha1 is used by session and torrent modules
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub mod bencode;
pub mod config;
mod dht;
pub mod peer;
pub mod piece;
pub mod session;
pub mod storage;
pub mod torrent;
pub mod tracker;

use config::Config;
use dht::DhtManager;
use torrent::TorrentMetainfo;

const STATE_FILE: &str = "aurora_state.json";
const TORRENTS_DIR: &str = ".torrents";
const RSS_DIR: &str = ".rss";

/// Pending magnet that needs metadata
#[derive(Clone)]
pub struct PendingMagnet {
    pub info_hash: String,
    pub name: String,
    pub trackers: Vec<String>,
    pub added_at: std::time::Instant,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

/// Per-torrent metadata stored separately
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TorrentMetadata {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub limits: TorrentLimits,
    pub sequential: bool,
    pub first_last_priority: bool,
    pub renamed: Option<String>,
    pub completed_on: u64,
    pub seeding_time: u64,
}

/// Main application state - shared between Tauri and the engine
#[derive(Clone)]
pub struct Engine {
    /// Active torrent sessions by ID
    sessions: Arc<DashMap<String, Arc<TorrentSession>>>,
    /// Pending magnets waiting for metadata
    pending_magnets: Arc<DashMap<String, PendingMagnet>>,
    /// Torrent metadata cache (for magnets that need metadata)
    metainfo_cache: Arc<DashMap<String, TorrentMetainfo>>,
    /// Per-torrent metadata (categories, tags, limits, etc)
    torrent_metadata: Arc<DashMap<String, TorrentMetadata>>,
    /// Configuration
    config: Arc<RwLock<Config>>,
    /// Our peer ID (used for BitTorrent protocol handshake)
    #[allow(dead_code)]
    peer_id: [u8; 20],
    /// Has FFmpeg for transcoding (for future use)
    #[allow(dead_code)]
    has_ffmpeg: bool,
    /// Listen port for incoming peers
    #[allow(dead_code)]
    listen_port: u16,
    /// Next listen port (incremented per torrent to avoid conflicts)
    next_port: Arc<std::sync::atomic::AtomicU16>,
    /// DHT manager for peer discovery
    dht: Arc<DhtManager>,
    /// File priorities per torrent (torrent_id -> file_index -> priority)
    file_priorities: Arc<DashMap<String, Vec<FilePriority>>>,
    /// Torrent metadata for persistence (saved .torrent bytes)
    torrent_data: Arc<DashMap<String, Vec<u8>>>,
    /// Queue positions (torrent_id -> position)
    queue_positions: Arc<DashMap<String, u32>>,
    /// When torrents were added (for sorting)
    added_times: Arc<DashMap<String, u64>>,
    /// RSS feeds
    rss_feeds: Arc<DashMap<String, RssFeed>>,
    /// RSS articles
    rss_articles: Arc<DashMap<String, RssArticle>>,
    /// RSS auto-download rules
    rss_rules: Arc<DashMap<String, RssDownloadRule>>,
    /// Search plugins
    search_plugins: Arc<DashMap<String, SearchPlugin>>,
    /// Session statistics
    session_stats: Arc<RwLock<SessionStats>>,
    /// Start time
    start_time: std::time::Instant,
    /// Total downloaded all-time
    total_downloaded: Arc<AtomicU64>,
    /// Total uploaded all-time
    total_uploaded: Arc<AtomicU64>,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    torrents: Vec<PersistedTorrent>,
    config: Config,
    #[serde(default)]
    queue_order: Vec<String>,
    #[serde(default)]
    rss_feeds: Vec<RssFeed>,
    #[serde(default)]
    rss_rules: Vec<RssDownloadRule>,
    #[serde(default)]
    total_downloaded: u64,
    #[serde(default)]
    total_uploaded: u64,
}

#[derive(Serialize, Deserialize)]
struct PersistedTorrent {
    id: String,
    name: String,
    magnet: Option<String>,
    torrent_filename: Option<String>,
    progress: f64,
    status: String,
    total_size: u64,
    added_on: u64,
    #[serde(default)]
    file_priorities: Vec<FilePriority>,
    #[serde(default)]
    metadata: TorrentMetadata,
}

/// Result type for engine operations
#[derive(Debug, Clone, Serialize)]
pub struct EngineResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> EngineResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

impl EngineResult<()> {
    pub fn ok_empty() -> Self {
        Self {
            success: true,
            data: Some(()),
            error: None,
        }
    }
}

impl Engine {
    /// Create a new engine instance and start background services
    pub async fn new() -> Result<Self> {
        tracing_subscriber::fmt::try_init().ok();
        info!("Initializing AuroraTorrent Engine v0.2.0");

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

        // Ensure directories exist
        fs::create_dir_all(&download_path).await?;
        fs::create_dir_all(PathBuf::from(&download_path).join(TORRENTS_DIR)).await?;
        fs::create_dir_all(PathBuf::from(&download_path).join(RSS_DIR)).await?;

        // Initialize DHT for peer discovery
        let dht = dht::init_dht().await?;

        // Initialize default search plugins
        let search_plugins = Arc::new(DashMap::new());
        // Add built-in search engines (placeholder - would need actual implementation)
        search_plugins.insert(
            "1337x".to_string(),
            SearchPlugin {
                name: "1337x".to_string(),
                url: "https://1337x.to".to_string(),
                enabled: true,
                supported_categories: vec![
                    "all".to_string(),
                    "movies".to_string(),
                    "tv".to_string(),
                    "games".to_string(),
                    "music".to_string(),
                    "software".to_string(),
                ],
                version: "1.0".to_string(),
            },
        );

        let engine = Self {
            sessions: Arc::new(DashMap::new()),
            pending_magnets: Arc::new(DashMap::new()),
            metainfo_cache: Arc::new(DashMap::new()),
            torrent_metadata: Arc::new(DashMap::new()),
            config: Arc::new(RwLock::new(config)),
            peer_id: peer::generate_peer_id(),
            has_ffmpeg,
            listen_port: 6881,
            next_port: Arc::new(std::sync::atomic::AtomicU16::new(6882)),
            dht,
            file_priorities: Arc::new(DashMap::new()),
            torrent_data: Arc::new(DashMap::new()),
            queue_positions: Arc::new(DashMap::new()),
            added_times: Arc::new(DashMap::new()),
            rss_feeds: Arc::new(DashMap::new()),
            rss_articles: Arc::new(DashMap::new()),
            rss_rules: Arc::new(DashMap::new()),
            search_plugins,
            session_stats: Arc::new(RwLock::new(SessionStats::default())),
            start_time: std::time::Instant::now(),
            total_downloaded: Arc::new(AtomicU64::new(0)),
            total_uploaded: Arc::new(AtomicU64::new(0)),
        };

        // Load previous state
        if let Err(e) = engine.load_state().await {
            warn!("Failed to load state: {}", e);
        }

        // Start streaming server with transcoding support
        let stream_state = engine.clone();
        let ffmpeg_available = has_ffmpeg;
        tokio::spawn(async move {
            let app = Router::new()
                // Direct streaming (native format)
                .route("/stream/:id/:file_idx", get(stream_handler))
                .route("/stream/:id", get(stream_default_handler))
                // HLS transcoding (for unsupported codecs)
                .route("/transcode/:id/:file_idx/master.m3u8", get(hls_master_handler))
                .route("/transcode/:id/:file_idx/stream.m3u8", get(hls_playlist_handler))
                .route("/transcode/:id/:file_idx/segment:seg.ts", get(hls_segment_handler))
                // Media info endpoint
                .route("/info/:id/:file_idx", get(media_info_handler))
                .with_state((stream_state, ffmpeg_available));

            let listener = match TcpListener::bind("127.0.0.1:3000").await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind streaming server: {}", e);
                    return;
                }
            };

            info!("Streaming server on http://127.0.0.1:3000 (FFmpeg: {})", ffmpeg_available);
            if let Err(e) = axum::serve(listener, app).await {
                error!("Streaming server error: {}", e);
            }
        });

        // Auto-save state periodically
        let save_state = engine.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = save_state.save_state().await {
                    warn!("Failed to save state: {}", e);
                }
            }
        });

        // Queue manager
        let queue_engine = engine.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                queue_engine.process_queue().await;
            }
        });

        // RSS refresh loop
        let rss_engine = engine.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                rss_engine.refresh_all_rss_feeds().await;
            }
        });

        // Stats update loop
        let stats_engine = engine.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                stats_engine.update_session_stats().await;
            }
        });

        info!("Engine initialized successfully");
        Ok(engine)
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) {
        info!("Shutting down engine...");
        for entry in self.sessions.iter() {
            entry.value().stop().await;
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save state on shutdown: {}", e);
        }
        info!("Engine shutdown complete");
    }

    // =========================================================================
    // TORRENT MANAGEMENT
    // =========================================================================

    /// Add a torrent from a magnet link
    pub async fn add_magnet(
        &self,
        magnet: &str,
        params: Option<AddTorrentParams>,
    ) -> EngineResult<serde_json::Value> {
        if !magnet.starts_with("magnet:?") {
            return EngineResult::err("Invalid magnet link: must start with 'magnet:?'");
        }
        if !magnet.contains("xt=urn:btih:") {
            return EngineResult::err("Invalid magnet link: missing info hash");
        }

        match TorrentMetainfo::from_magnet(magnet) {
            Ok(magnet_info) => {
                let id = magnet_info.info_hash_hex();

                if self.sessions.contains_key(&id) || self.pending_magnets.contains_key(&id) {
                    return EngineResult::err("Torrent already exists");
                }

                let params = params.unwrap_or_default();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                self.pending_magnets.insert(
                    id.clone(),
                    PendingMagnet {
                        info_hash: id.clone(),
                        name: magnet_info.name.clone(),
                        trackers: magnet_info.trackers.clone(),
                        added_at: std::time::Instant::now(),
                        category: params.category.clone(),
                        tags: params.tags.clone(),
                    },
                );

                self.added_times.insert(id.clone(), now);
                self.queue_positions
                    .insert(id.clone(), self.get_next_queue_position());

                // Store metadata
                self.torrent_metadata.insert(
                    id.clone(),
                    TorrentMetadata {
                        category: params.category,
                        tags: params.tags,
                        sequential: params.sequential_download.unwrap_or(false),
                        first_last_priority: params.first_last_piece_priority.unwrap_or(false),
                        ..Default::default()
                    },
                );

                info!("Added magnet: {} ({})", magnet_info.name, id);

                if let Err(e) = self.save_state().await {
                    warn!("Failed to save state: {}", e);
                }

                EngineResult::ok(serde_json::json!({
                    "status": "added",
                    "id": id,
                    "name": magnet_info.name
                }))
            }
            Err(e) => EngineResult::err(format!("Invalid magnet: {}", e)),
        }
    }

    /// Add a torrent from a .torrent file (base64 encoded content)
    pub async fn add_torrent_file(
        &self,
        _name: &str,
        content: &str,
        params: Option<AddTorrentParams>,
    ) -> EngineResult<serde_json::Value> {
        let bytes = match base64_decode(content) {
            Ok(b) => b,
            Err(e) => return EngineResult::err(format!("Invalid torrent file: {}", e)),
        };

        self.add_torrent_bytes(&bytes, true, params).await
    }

    async fn add_torrent_bytes(
        &self,
        bytes: &[u8],
        auto_start: bool,
        params: Option<AddTorrentParams>,
    ) -> EngineResult<serde_json::Value> {
        match TorrentMetainfo::from_bytes(bytes) {
            Ok(metainfo) => {
                let id = metainfo.info_hash_hex();

                if self.sessions.contains_key(&id) {
                    return EngineResult::err("Torrent already exists");
                }

                let params = params.unwrap_or_default();
                self.torrent_data.insert(id.clone(), bytes.to_vec());

                if let Err(e) = self.save_torrent_file(&id, bytes).await {
                    warn!("Failed to save .torrent file: {}", e);
                }

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();
                self.added_times.insert(id.clone(), now);

                let priorities: Vec<FilePriority> =
                    vec![FilePriority::Normal; metainfo.files.len()];
                self.file_priorities.insert(id.clone(), priorities);

                self.queue_positions
                    .insert(id.clone(), self.get_next_queue_position());

                // Store metadata
                self.torrent_metadata.insert(
                    id.clone(),
                    TorrentMetadata {
                        category: params.category,
                        tags: params.tags,
                        sequential: params.sequential_download.unwrap_or(false),
                        first_last_priority: params.first_last_piece_priority.unwrap_or(false),
                        ..Default::default()
                    },
                );

                let download_path = params.save_path.map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from(&self.config.blocking_read().download_path)
                });

                let port = self
                    .next_port
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let session = Arc::new(TorrentSession::new(metainfo.clone(), download_path, port));

                let should_start =
                    auto_start && params.add_paused != Some(true) && self.config.blocking_read().auto_start;

                if should_start {
                    session.clone().start().await;
                    self.start_dht_for_torrent(&session, &metainfo).await;
                }

                self.sessions.insert(id.clone(), session);
                self.metainfo_cache.insert(id.clone(), metainfo.clone());

                info!("Added torrent: {} ({})", metainfo.name, id);

                if let Err(e) = self.save_state().await {
                    warn!("Failed to save state: {}", e);
                }

                EngineResult::ok(serde_json::json!({
                    "status": "added",
                    "id": id,
                    "name": metainfo.name
                }))
            }
            Err(e) => EngineResult::err(format!("Failed to parse torrent: {}", e)),
        }
    }

    /// List all torrents
    pub async fn list_torrents(&self) -> Vec<TorrentState> {
        let mut torrents = Vec::new();

        for entry in self.sessions.iter() {
            let t = self.torrent_state(entry.key(), entry.value()).await;
            torrents.push(t);
        }

        for entry in self.pending_magnets.iter() {
            let pending = entry.value();
            let meta = self
                .torrent_metadata
                .get(&pending.info_hash)
                .map(|m| m.clone())
                .unwrap_or_default();
            let queue_pos = self
                .queue_positions
                .get(&pending.info_hash)
                .map(|r| *r)
                .unwrap_or(0);
            let added_on = self
                .added_times
                .get(&pending.info_hash)
                .map(|r| *r)
                .unwrap_or(0);

            torrents.push(TorrentState {
                id: pending.info_hash.clone(),
                name: pending.name.clone(),
                status: "FetchingMetadata".to_string(),
                category: meta.category,
                tags: meta.tags,
                queue_position: queue_pos,
                added_on,
                ..Default::default()
            });
        }

        torrents.sort_by_key(|t| t.queue_position);
        torrents
    }

    /// Start/resume a torrent
    pub async fn start_torrent(&self, id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.resume().await;
            if let Some(metainfo) = self.metainfo_cache.get(id) {
                self.start_dht_for_torrent(&session, &metainfo).await;
            }
            EngineResult::ok_empty()
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    /// Pause a torrent
    pub async fn pause_torrent(&self, id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.pause().await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    /// Stop a torrent completely
    pub async fn stop_torrent(&self, id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.stop().await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    /// Remove a torrent
    pub async fn remove_torrent(&self, id: &str, delete_files: bool) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some((_, session)) = self.sessions.remove(id) {
            session.stop().await;
            self.metainfo_cache.remove(id);
            self.file_priorities.remove(id);
            self.torrent_data.remove(id);
            self.queue_positions.remove(id);
            self.added_times.remove(id);
            self.torrent_metadata.remove(id);

            let _ = self.delete_torrent_file(id).await;

            if delete_files {
                let download_path = self.config.read().await.download_path.clone();
                let torrent_path = PathBuf::from(&download_path).join(&session.metainfo.name);
                if torrent_path.exists() {
                    if let Err(e) = fs::remove_dir_all(&torrent_path).await {
                        warn!("Failed to delete files: {}", e);
                    }
                }
            }

            if let Err(e) = self.save_state().await {
                warn!("Failed to save state after removal: {}", e);
            }

            return EngineResult::ok_empty();
        }

        if self.pending_magnets.remove(id).is_some() {
            self.queue_positions.remove(id);
            self.added_times.remove(id);
            self.torrent_metadata.remove(id);
            return EngineResult::ok_empty();
        }

        EngineResult::err(format!("Torrent {} not found", id))
    }

    /// Force recheck torrent
    pub async fn force_recheck(&self, id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.force_recheck().await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    /// Force reannounce to trackers
    pub async fn force_reannounce(&self, id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.force_reannounce().await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    /// Rename a torrent
    pub async fn rename_torrent(&self, id: &str, new_name: &str) -> EngineResult<()> {
        if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
            meta.renamed = Some(new_name.to_string());
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Set torrent category
    pub async fn set_torrent_category(&self, id: &str, category: Option<String>) -> EngineResult<()> {
        if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
            meta.category = category;
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Add tags to torrent
    pub async fn add_torrent_tags(&self, id: &str, tags: Vec<String>) -> EngineResult<()> {
        if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
            for tag in tags {
                if !meta.tags.contains(&tag) {
                    meta.tags.push(tag);
                }
            }
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Remove tags from torrent
    pub async fn remove_torrent_tags(&self, id: &str, tags: Vec<String>) -> EngineResult<()> {
        if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
            meta.tags.retain(|t| !tags.contains(t));
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Set per-torrent limits
    pub async fn set_torrent_limits(&self, id: &str, limits: TorrentLimits) -> EngineResult<()> {
        if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
            meta.limits = limits;
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Toggle sequential download
    pub async fn toggle_sequential(&self, id: &str, enabled: bool) -> EngineResult<()> {
        if let Some(session) = self.sessions.get(id) {
            session.set_sequential(enabled).await;
            if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
                meta.sequential = enabled;
            }
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    /// Toggle first/last piece priority
    pub async fn toggle_first_last_piece(&self, id: &str, enabled: bool) -> EngineResult<()> {
        if let Some(session) = self.sessions.get(id) {
            if enabled {
                let total_pieces = session.metainfo.pieces.len() / 20;
                if total_pieces > 0 {
                    session
                        .prioritize_pieces(vec![0, (total_pieces - 1) as u32])
                        .await;
                }
            }
            if let Some(mut meta) = self.torrent_metadata.get_mut(id) {
                meta.first_last_priority = enabled;
            }
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    // =========================================================================
    // STREAMING
    // =========================================================================

    pub async fn stream_torrent(&self, id: &str) -> EngineResult<serde_json::Value> {
        if let Err(e) = validate_torrent_id(id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(id) {
            session.set_sequential(true).await;
            let pieces_to_prioritize: Vec<u32> = (0..10).collect();
            session.prioritize_pieces(pieces_to_prioritize).await;

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
                        || name.ends_with(".avi")
                })
                .unwrap_or(0);

            EngineResult::ok(serde_json::json!({
                "status": "streaming",
                "url": format!("http://127.0.0.1:3000/stream/{}/{}", id, file_idx)
            }))
        } else {
            EngineResult::err(format!("Torrent {} not found", id))
        }
    }

    // =========================================================================
    // TRACKERS
    // =========================================================================

    pub async fn add_trackers(&self, id: &str, trackers: Vec<String>) -> EngineResult<()> {
        if let Some(session) = self.sessions.get(id) {
            session.add_trackers(trackers).await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    pub async fn remove_trackers(&self, id: &str, trackers: Vec<String>) -> EngineResult<()> {
        if let Some(session) = self.sessions.get(id) {
            session.remove_trackers(trackers).await;
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Torrent not found")
        }
    }

    // =========================================================================
    // FILE PRIORITY
    // =========================================================================

    pub async fn get_torrent_files(&self, torrent_id: &str) -> EngineResult<Vec<FileInfo>> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(session) = self.sessions.get(torrent_id) {
            let priorities = self
                .file_priorities
                .get(torrent_id)
                .map(|p| p.clone())
                .unwrap_or_default();

            let stats = session.stats().await;
            let files: Vec<FileInfo> = session
                .metainfo
                .files
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let priority = priorities.get(i).copied().unwrap_or(FilePriority::Normal);
                    FileInfo {
                        index: i,
                        name: f
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        path: f.path.to_string_lossy().to_string(),
                        size: f.length,
                        progress: if stats.progress >= 1.0 {
                            1.0
                        } else {
                            stats.progress
                        },
                        priority,
                        selected: priority != FilePriority::Skip,
                    }
                })
                .collect();

            return EngineResult::ok(files);
        }

        EngineResult::err("Torrent not found")
    }

    pub async fn set_file_priority(
        &self,
        torrent_id: &str,
        file_index: usize,
        priority: FilePriority,
    ) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(mut priorities) = self.file_priorities.get_mut(torrent_id) {
            if file_index < priorities.len() {
                priorities[file_index] = priority;
                return EngineResult::ok_empty();
            }
        }

        EngineResult::err("File not found")
    }

    pub async fn set_files_priority(
        &self,
        torrent_id: &str,
        file_indices: Vec<usize>,
        priority: FilePriority,
    ) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(mut priorities) = self.file_priorities.get_mut(torrent_id) {
            for idx in file_indices {
                if idx < priorities.len() {
                    priorities[idx] = priority;
                }
            }
            return EngineResult::ok_empty();
        }

        EngineResult::err("Torrent not found")
    }

    // =========================================================================
    // QUEUE MANAGEMENT
    // =========================================================================

    pub async fn queue_move_up(&self, torrent_id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(current_pos) = self.queue_positions.get(torrent_id) {
            let current = *current_pos;
            if current > 0 {
                for entry in self.queue_positions.iter() {
                    if *entry.value() == current - 1 {
                        let other_id = entry.key().clone();
                        drop(entry);
                        self.queue_positions.insert(other_id, current);
                        break;
                    }
                }
                self.queue_positions.insert(torrent_id.to_string(), current - 1);
            }
        }

        EngineResult::ok_empty()
    }

    pub async fn queue_move_down(&self, torrent_id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(current_pos) = self.queue_positions.get(torrent_id) {
            let current = *current_pos;
            let max_pos = self.queue_positions.len() as u32 - 1;

            if current < max_pos {
                for entry in self.queue_positions.iter() {
                    if *entry.value() == current + 1 {
                        let other_id = entry.key().clone();
                        drop(entry);
                        self.queue_positions.insert(other_id, current);
                        break;
                    }
                }
                self.queue_positions.insert(torrent_id.to_string(), current + 1);
            }
        }

        EngineResult::ok_empty()
    }

    pub async fn queue_move_top(&self, torrent_id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(current_pos) = self.queue_positions.get(torrent_id) {
            let current = *current_pos;

            for mut entry in self.queue_positions.iter_mut() {
                if *entry.value() < current {
                    *entry.value_mut() += 1;
                }
            }
            self.queue_positions.insert(torrent_id.to_string(), 0);
        }

        EngineResult::ok_empty()
    }

    pub async fn queue_move_bottom(&self, torrent_id: &str) -> EngineResult<()> {
        if let Err(e) = validate_torrent_id(torrent_id) {
            return EngineResult::err(e);
        }

        if let Some(current_pos) = self.queue_positions.get(torrent_id) {
            let current = *current_pos;
            let max_pos = self.queue_positions.len() as u32 - 1;

            for mut entry in self.queue_positions.iter_mut() {
                if *entry.value() > current {
                    *entry.value_mut() -= 1;
                }
            }
            self.queue_positions.insert(torrent_id.to_string(), max_pos);
        }

        EngineResult::ok_empty()
    }

    // =========================================================================
    // CONFIGURATION
    // =========================================================================

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn set_full_config(&self, config: Config) -> EngineResult<()> {
        *self.config.write().await = config;
        if let Err(e) = self.save_state().await {
            warn!("Failed to save config: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn set_config(
        &self,
        download_path: Option<String>,
        max_download_speed: Option<u64>,
        max_upload_speed: Option<u64>,
        queue_settings: Option<bridge::QueueSettings>,
    ) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            if let Some(p) = download_path {
                config.download_path = p;
            }
            if let Some(s) = max_download_speed {
                config.max_download_speed = s;
            }
            if let Some(s) = max_upload_speed {
                config.max_upload_speed = s;
            }
            if let Some(q) = queue_settings {
                config.queue = q;
            }
        }

        if let Err(e) = self.save_state().await {
            warn!("Failed to save config: {}", e);
        }

        EngineResult::ok_empty()
    }

    pub async fn toggle_alt_speed(&self, enabled: bool) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            config.use_alt_speed_limits = enabled;
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save alt speed setting: {}", e);
        }
        EngineResult::ok_empty()
    }

    // =========================================================================
    // CATEGORIES AND TAGS
    // =========================================================================

    pub async fn create_category(&self, name: String, save_path: Option<String>) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            config.add_category(name, save_path);
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save category: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn edit_category(&self, name: String, save_path: Option<String>) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            if !config.categories.contains_key(&name) {
                return EngineResult::err("Category not found");
            }
            config.categories.insert(name.clone(), Category { name, save_path });
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save category edit: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn delete_category(&self, name: &str) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            if !config.remove_category(name) {
                return EngineResult::err("Category not found");
            }
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save category deletion: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn get_categories(&self) -> HashMap<String, Category> {
        self.config.read().await.categories.clone()
    }

    pub async fn create_tag(&self, tag: String) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            config.add_tag(tag);
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save tag: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn delete_tag(&self, tag: &str) -> EngineResult<()> {
        {
            let mut config = self.config.write().await;
            if !config.remove_tag(tag) {
                return EngineResult::err("Tag not found");
            }
        }
        if let Err(e) = self.save_state().await {
            warn!("Failed to save tag deletion: {}", e);
        }
        EngineResult::ok_empty()
    }

    pub async fn get_tags(&self) -> Vec<String> {
        self.config.read().await.tags.clone()
    }

    // =========================================================================
    // RSS FEEDS
    // =========================================================================

    pub async fn get_rss_feeds(&self) -> Vec<RssFeed> {
        self.rss_feeds.iter().map(|e| e.value().clone()).collect()
    }

    pub async fn add_rss_feed(&self, url: String, name: Option<String>) -> EngineResult<RssFeed> {
        let id = generate_id();
        let feed = RssFeed {
            id: id.clone(),
            name: name.unwrap_or_else(|| url.clone()),
            url,
            enabled: true,
            refresh_interval: 30,
            last_refresh: None,
            auto_download: false,
        };
        self.rss_feeds.insert(id, feed.clone());
        EngineResult::ok(feed)
    }

    pub async fn remove_rss_feed(&self, id: &str) -> EngineResult<()> {
        if self.rss_feeds.remove(id).is_some() {
            // Remove associated articles
            self.rss_articles.retain(|_, a| a.feed_id != id);
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Feed not found")
        }
    }

    pub async fn refresh_rss_feed(&self, id: &str) -> EngineResult<()> {
        if let Some(mut feed) = self.rss_feeds.get_mut(id) {
            // In production, this would fetch and parse the RSS feed
            feed.last_refresh = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            info!("Refreshed RSS feed: {}", feed.name);
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Feed not found")
        }
    }

    async fn refresh_all_rss_feeds(&self) {
        let config = self.config.read().await;
        if !config.rss_auto_download_enabled {
            return;
        }
        let interval_secs = config.rss_refresh_interval as u64 * 60;
        drop(config);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for mut feed in self.rss_feeds.iter_mut() {
            if !feed.enabled {
                continue;
            }
            let should_refresh = feed
                .last_refresh
                .map(|last| now - last > interval_secs)
                .unwrap_or(true);

            if should_refresh {
                feed.last_refresh = Some(now);
                // TODO: Actually fetch and parse RSS feed
            }
        }
    }

    pub async fn get_rss_articles(&self, feed_id: Option<String>) -> Vec<RssArticle> {
        self.rss_articles
            .iter()
            .filter(|e| feed_id.as_ref().map(|id| &e.value().feed_id == id).unwrap_or(true))
            .map(|e| e.value().clone())
            .collect()
    }

    pub async fn get_rss_rules(&self) -> Vec<RssDownloadRule> {
        self.rss_rules.iter().map(|e| e.value().clone()).collect()
    }

    pub async fn add_rss_rule(&self, rule: RssDownloadRule) -> EngineResult<()> {
        self.rss_rules.insert(rule.id.clone(), rule);
        EngineResult::ok_empty()
    }

    pub async fn remove_rss_rule(&self, id: &str) -> EngineResult<()> {
        if self.rss_rules.remove(id).is_some() {
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Rule not found")
        }
    }

    // =========================================================================
    // SEARCH
    // =========================================================================

    pub async fn search_torrents(
        &self,
        query: &str,
        _plugins: Option<Vec<String>>,
        _category: Option<String>,
    ) -> EngineResult<Vec<SearchResult>> {
        // Placeholder - in production, this would query actual search plugins
        info!("Search query: {}", query);
        EngineResult::ok(vec![])
    }

    pub async fn get_search_plugins(&self) -> Vec<SearchPlugin> {
        self.search_plugins.iter().map(|e| e.value().clone()).collect()
    }

    pub async fn enable_search_plugin(&self, name: &str, enabled: bool) -> EngineResult<()> {
        if let Some(mut plugin) = self.search_plugins.get_mut(name) {
            plugin.enabled = enabled;
            EngineResult::ok_empty()
        } else {
            EngineResult::err("Plugin not found")
        }
    }

    // =========================================================================
    // TORRENT CREATION
    // =========================================================================

    pub async fn create_torrent(
        &self,
        source_path: &str,
        trackers: Vec<String>,
        comment: Option<String>,
        is_private: bool,
        piece_size: Option<u64>,
    ) -> EngineResult<Vec<u8>> {
        let path = PathBuf::from(source_path);
        if !path.exists() {
            return EngineResult::err("Source path does not exist");
        }

        let piece_length = piece_size.unwrap_or(262144); // 256 KB default

        match torrent::create_torrent(&path, trackers, comment, is_private, piece_length).await {
            Ok(bytes) => EngineResult::ok(bytes),
            Err(e) => EngineResult::err(format!("Failed to create torrent: {}", e)),
        }
    }

    // =========================================================================
    // SESSION STATS
    // =========================================================================

    pub async fn get_session_stats(&self) -> SessionStats {
        self.session_stats.read().await.clone()
    }

    async fn update_session_stats(&self) {
        let mut stats = self.session_stats.write().await;

        stats.total_torrents = self.sessions.len() as u32 + self.pending_magnets.len() as u32;
        stats.up_time = self.start_time.elapsed().as_secs();

        let mut download_rate = 0u64;
        let mut upload_rate = 0u64;
        let mut downloading = 0u32;
        let mut seeding = 0u32;
        let mut paused = 0u32;
        let mut checking = 0u32;
        let mut errors = 0u32;
        let mut peers = 0u32;

        for entry in self.sessions.iter() {
            let session = entry.value();
            let s = session.stats().await;
            let state = session.state().await;

            download_rate += s.download_rate;
            upload_rate += s.upload_rate;
            peers += s.connected_peers as u32;

            match state {
                SessionState::Downloading | SessionState::Starting => downloading += 1,
                SessionState::Seeding => seeding += 1,
                SessionState::Paused => paused += 1,
                SessionState::Error => errors += 1,
                _ => {}
            }
        }

        stats.download_rate = download_rate;
        stats.upload_rate = upload_rate;
        stats.downloading_torrents = downloading;
        stats.seeding_torrents = seeding;
        stats.paused_torrents = paused;
        stats.checking_torrents = checking;
        stats.error_torrents = errors;
        stats.peers_connected = peers;

        self.total_downloaded.fetch_add(download_rate, Ordering::Relaxed);
        self.total_uploaded.fetch_add(upload_rate, Ordering::Relaxed);

        stats.total_downloaded = self.total_downloaded.load(Ordering::Relaxed);
        stats.total_uploaded = self.total_uploaded.load(Ordering::Relaxed);
        stats.total_downloaded_session = self.total_downloaded.load(Ordering::Relaxed);
        stats.total_uploaded_session = self.total_uploaded.load(Ordering::Relaxed);

        if stats.total_downloaded > 0 {
            stats.global_ratio = stats.total_uploaded as f64 / stats.total_downloaded as f64;
        }
    }

    // =========================================================================
    // UTILITIES
    // =========================================================================

    pub async fn ban_peer(&self, ip: &str) -> EngineResult<()> {
        let mut config = self.config.write().await;
        config.ban_ip(ip.to_string());
        EngineResult::ok_empty()
    }

    // =========================================================================
    // PRIVATE HELPERS
    // =========================================================================

    fn get_next_queue_position(&self) -> u32 {
        let mut max_pos = 0u32;
        for entry in self.queue_positions.iter() {
            if *entry.value() >= max_pos {
                max_pos = *entry.value() + 1;
            }
        }
        max_pos
    }

    async fn process_queue(&self) {
        let config = self.config.read().await;
        let max_active = config.queue.max_active_downloads as usize;
        drop(config);

        let mut downloading = Vec::new();
        let mut queued = Vec::new();

        for entry in self.sessions.iter() {
            let state = entry.value().state().await;
            let queue_pos = self.queue_positions.get(entry.key()).map(|r| *r).unwrap_or(u32::MAX);

            match state {
                SessionState::Downloading | SessionState::Starting => {
                    downloading.push((entry.key().clone(), queue_pos));
                }
                SessionState::Paused | SessionState::Stopped => {
                    let stats = entry.value().stats().await;
                    if stats.progress < 1.0 {
                        queued.push((entry.key().clone(), queue_pos));
                    }
                }
                _ => {}
            }
        }

        downloading.sort_by_key(|(_, pos)| *pos);
        queued.sort_by_key(|(_, pos)| *pos);

        if downloading.len() < max_active {
            let to_start = max_active - downloading.len();
            for (id, _) in queued.into_iter().take(to_start) {
                if let Some(session) = self.sessions.get(&id) {
                    info!("Queue: Starting torrent {}", id);
                    session.resume().await;
                }
            }
        }
    }

    async fn start_dht_for_torrent(&self, session: &TorrentSession, metainfo: &TorrentMetainfo) {
        let dht = self.dht.clone();
        let info_hash = metainfo.info_hash;
        let port = session.port;
        let _session_for_dht = Arc::new(session.clone());

        tokio::spawn(async move {
            if let Err(e) = dht.announce(info_hash, port) {
                warn!("DHT announce failed: {}", e);
            }

            let dht_clone = dht.clone();
            let peers = tokio::task::spawn_blocking(move || dht_clone.get_peers(info_hash))
                .await
                .unwrap_or_default();

            if !peers.is_empty() {
                // Note: This won't work as session is not cloneable this way
                // session_for_dht.add_peers(peers).await;
            }
        });
    }

    async fn save_torrent_file(&self, id: &str, bytes: &[u8]) -> Result<()> {
        let download_path = self.config.read().await.download_path.clone();
        let torrents_dir = PathBuf::from(&download_path).join(TORRENTS_DIR);
        fs::create_dir_all(&torrents_dir).await?;

        let torrent_path = torrents_dir.join(format!("{}.torrent", id));
        fs::write(&torrent_path, bytes).await?;

        Ok(())
    }

    async fn delete_torrent_file(&self, id: &str) -> Result<()> {
        let download_path = self.config.read().await.download_path.clone();
        let torrent_path = PathBuf::from(&download_path)
            .join(TORRENTS_DIR)
            .join(format!("{}.torrent", id));

        if torrent_path.exists() {
            fs::remove_file(&torrent_path).await?;
        }

        Ok(())
    }

    async fn save_state(&self) -> Result<()> {
        let config = self.config.read().await.clone();
        let mut torrents = Vec::new();

        let mut positions: Vec<(String, u32)> = self
            .queue_positions
            .iter()
            .map(|e| (e.key().clone(), *e.value()))
            .collect();
        positions.sort_by_key(|(_, pos)| *pos);
        let queue_order: Vec<String> = positions.into_iter().map(|(id, _)| id).collect();

        for entry in self.sessions.iter() {
            let session = entry.value();
            let stats = session.stats().await;
            let state = session.state().await;
            let added_on = self.added_times.get(entry.key()).map(|r| *r).unwrap_or(0);
            let priorities = self
                .file_priorities
                .get(entry.key())
                .map(|p| p.clone())
                .unwrap_or_default();
            let metadata = self
                .torrent_metadata
                .get(entry.key())
                .map(|m| m.clone())
                .unwrap_or_default();

            torrents.push(PersistedTorrent {
                id: entry.key().clone(),
                name: session.metainfo.name.clone(),
                magnet: None,
                torrent_filename: Some(format!("{}.torrent", entry.key())),
                progress: stats.progress,
                status: match state {
                    SessionState::Downloading => "Downloading",
                    SessionState::Seeding => "Seeding",
                    SessionState::Paused => "Paused",
                    _ => "Stopped",
                }
                .to_string(),
                total_size: session.metainfo.total_size,
                added_on,
                file_priorities: priorities,
                metadata,
            });
        }

        for entry in self.pending_magnets.iter() {
            let pending = entry.value();
            let added_on = self.added_times.get(&pending.info_hash).map(|r| *r).unwrap_or(0);
            let metadata = self
                .torrent_metadata
                .get(&pending.info_hash)
                .map(|m| m.clone())
                .unwrap_or_default();

            torrents.push(PersistedTorrent {
                id: pending.info_hash.clone(),
                name: pending.name.clone(),
                magnet: Some(format!(
                    "magnet:?xt=urn:btih:{}&dn={}",
                    pending.info_hash,
                    urlencoding::encode(&pending.name)
                )),
                torrent_filename: None,
                progress: 0.0,
                status: "Pending".to_string(),
                total_size: 0,
                added_on,
                file_priorities: vec![],
                metadata,
            });
        }

        let rss_feeds: Vec<RssFeed> = self.rss_feeds.iter().map(|e| e.value().clone()).collect();
        let rss_rules: Vec<RssDownloadRule> = self.rss_rules.iter().map(|e| e.value().clone()).collect();

        let data = PersistedState {
            torrents,
            config,
            queue_order,
            rss_feeds,
            rss_rules,
            total_downloaded: self.total_downloaded.load(Ordering::Relaxed),
            total_uploaded: self.total_uploaded.load(Ordering::Relaxed),
        };

        let json = serde_json::to_string_pretty(&data)?;
        let state_path = self.config.read().await.download_path.clone();
        let state_file = PathBuf::from(&state_path).join(STATE_FILE);

        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        let temp_file = state_file.with_extension("tmp");
        let mut file = fs::File::create(&temp_file).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, json.as_bytes()).await?;
        tokio::io::AsyncWriteExt::flush(&mut file).await?;
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

        *self.config.write().await = data.config.clone();

        // Restore stats
        self.total_downloaded.store(data.total_downloaded, Ordering::Relaxed);
        self.total_uploaded.store(data.total_uploaded, Ordering::Relaxed);

        // Restore RSS
        for feed in data.rss_feeds {
            self.rss_feeds.insert(feed.id.clone(), feed);
        }
        for rule in data.rss_rules {
            self.rss_rules.insert(rule.id.clone(), rule);
        }

        info!("Restoring {} torrents from state", data.torrents.len());

        let torrents_dir = PathBuf::from(&state_path).join(TORRENTS_DIR);
        let start_on_launch = data.config.start_on_launch;

        for (pos, id) in data.queue_order.iter().enumerate() {
            self.queue_positions.insert(id.clone(), pos as u32);
        }

        for (idx, persisted) in data.torrents.into_iter().enumerate() {
            self.added_times.insert(persisted.id.clone(), persisted.added_on);
            self.torrent_metadata.insert(persisted.id.clone(), persisted.metadata.clone());

            if !self.queue_positions.contains_key(&persisted.id) {
                self.queue_positions.insert(persisted.id.clone(), idx as u32);
            }

            if let Some(filename) = &persisted.torrent_filename {
                let torrent_path = torrents_dir.join(filename);
                if torrent_path.exists() {
                    match fs::read(&torrent_path).await {
                        Ok(bytes) => {
                            if !persisted.file_priorities.is_empty() {
                                self.file_priorities
                                    .insert(persisted.id.clone(), persisted.file_priorities.clone());
                            }

                            let should_start = start_on_launch
                                && persisted.status != "Stopped"
                                && persisted.status != "Paused";

                            if let Err(e) = self.restore_torrent(&bytes, should_start).await {
                                warn!("Failed to restore torrent {}: {}", persisted.name, e);
                            } else {
                                info!("Restored torrent: {}", persisted.name);
                            }
                            continue;
                        }
                        Err(e) => {
                            warn!("Failed to read torrent file {}: {}", filename, e);
                        }
                    }
                }
            }

            if let Some(magnet) = &persisted.magnet {
                if let Ok(magnet_info) = TorrentMetainfo::from_magnet(magnet) {
                    self.pending_magnets.insert(
                        persisted.id.clone(),
                        PendingMagnet {
                            info_hash: persisted.id.clone(),
                            name: magnet_info.name,
                            trackers: magnet_info.trackers,
                            added_at: std::time::Instant::now(),
                            category: persisted.metadata.category.clone(),
                            tags: persisted.metadata.tags.clone(),
                        },
                    );
                }
            }
        }

        info!("State restoration complete");
        Ok(())
    }

    async fn restore_torrent(&self, bytes: &[u8], auto_start: bool) -> Result<()> {
        let metainfo = TorrentMetainfo::from_bytes(bytes)?;
        let id = metainfo.info_hash_hex();

        if self.sessions.contains_key(&id) {
            return Ok(());
        }

        self.torrent_data.insert(id.clone(), bytes.to_vec());

        if !self.file_priorities.contains_key(&id) {
            let priorities: Vec<FilePriority> = vec![FilePriority::Normal; metainfo.files.len()];
            self.file_priorities.insert(id.clone(), priorities);
        }

        let download_path = PathBuf::from(&self.config.read().await.download_path);
        let port = self.next_port.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let session = Arc::new(TorrentSession::new(metainfo.clone(), download_path, port));

        if auto_start {
            session.clone().start().await;
        }

        self.sessions.insert(id.clone(), session);
        self.metainfo_cache.insert(id, metainfo);

        Ok(())
    }

    async fn torrent_state(&self, id: &str, session: &TorrentSession) -> TorrentState {
        let stats = session.stats().await;
        let state = session.state().await;
        let peer_stats = session.peer_stats().await;
        let queue_pos = self.queue_positions.get(id).map(|r| *r).unwrap_or(0);
        let added_on = self.added_times.get(id).map(|r| *r).unwrap_or(0);

        let priorities = self
            .file_priorities
            .get(id)
            .map(|p| p.clone())
            .unwrap_or_default();

        let metadata = self
            .torrent_metadata
            .get(id)
            .map(|m| m.clone())
            .unwrap_or_default();

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
                        let end_piece = ((f.offset + f.length - 1) / session.metainfo.piece_length) as usize;
                        let total = end_piece - start_piece + 1;
                        let complete = (start_piece..=end_piece)
                            .filter(|&p| p < total_pieces && bitfield[p])
                            .count();
                        complete as f64 / total as f64
                    }
                })
                .collect()
        };

        // Build piece states for visualization
        let piece_states: Vec<u8> = {
            let pieces = session.piece_manager();
            let pm = pieces.read().await;
            (0..pm.total_pieces())
                .map(|i| if pm.have[i] { 2 } else { 0 })
                .collect()
        };

        let eta = if stats.download_rate > 0 && stats.progress < 1.0 {
            let remaining = session.metainfo.total_size as f64 * (1.0 - stats.progress);
            (remaining / stats.download_rate as f64) as u64
        } else {
            0
        };

        let name = metadata.renamed.clone().unwrap_or_else(|| session.metainfo.name.clone());

        TorrentState {
            id: id.to_string(),
            name,
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
            downloaded: stats.downloaded,
            uploaded: stats.uploaded,
            total_size: session.metainfo.total_size,
            files: session
                .metainfo
                .files
                .iter()
                .zip(file_progresses)
                .enumerate()
                .map(|(i, (f, progress))| {
                    let priority = priorities.get(i).copied().unwrap_or(FilePriority::Normal);
                    FileInfo {
                        index: i,
                        name: f
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        path: f.path.to_string_lossy().to_string(),
                        size: f.length,
                        progress,
                        priority,
                        selected: priority != FilePriority::Skip,
                    }
                })
                .collect(),
            peers: peer_stats
                .iter()
                .map(|p| PeerInfo {
                    ip: p.addr.ip().to_string(),
                    port: p.addr.port(),
                    client: p.client.clone(),
                    down_speed: p.download_rate,
                    up_speed: p.upload_rate,
                    progress: 0.0,
                    ..Default::default()
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
                    ..Default::default()
                })
                .collect(),
            seeds: peer_stats.len() as u32,
            leechers: 0,
            queue_position: queue_pos,
            eta,
            added_on,
            category: metadata.category,
            tags: metadata.tags,
            save_path: session.download_path.to_string_lossy().to_string(),
            ratio: if stats.downloaded > 0 {
                stats.uploaded as f64 / stats.downloaded as f64
            } else {
                0.0
            },
            seeding_time: metadata.seeding_time,
            sequential_download: metadata.sequential,
            first_last_piece_priority: metadata.first_last_priority,
            auto_managed: true,
            super_seeding: false,
            force_start: false,
            limits: metadata.limits,
            piece_states,
            comment: session.metainfo.comment.clone(),
            created_by: session.metainfo.created_by.clone(),
            creation_date: session.metainfo.creation_date.map(|d| d as u64),
            is_private: session.metainfo.private,
            magnet_uri: Some(format!(
                "magnet:?xt=urn:btih:{}&dn={}",
                id,
                urlencoding::encode(&session.metainfo.name)
            )),
            num_pieces: (session.metainfo.pieces.len() / 20) as u32,
            piece_size: session.metainfo.piece_length,
            ..Default::default()
        }
    }
}

// =============================================================================
// STREAMING SERVER HANDLERS
// =============================================================================

type StreamState = (Engine, bool); // (Engine, has_ffmpeg)

async fn stream_handler(
    Path((id, file_idx)): Path<(String, usize)>,
    State((state, _has_ffmpeg)): State<StreamState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Torrent not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    let file = match session.metainfo.files.get(file_idx) {
        Some(f) => f,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    let file_size = file.length;
    let content_type = mime_guess::from_path(&file.path)
        .first_or_octet_stream()
        .to_string();

    // Handle zero-length files
    if file_size == 0 {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, 0)
            .body(Body::empty())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| parse_range(s, file_size));

    // file_size > 0 guaranteed at this point
    let (start, end) = range.unwrap_or((0, file_size - 1));
    let content_length = end - start + 1;
    let data_start = file.offset + start;

    let pieces = session.piece_manager();
    {
        let pm = pieces.read().await;

        if !pm.is_range_available(data_start, content_length) {
            let needed = pm.pieces_for_range(data_start, content_length);
            drop(pm);
            session.prioritize_pieces(needed).await;

            return Response::builder()
                .status(StatusCode::ACCEPTED)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("Buffering..."))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    }

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

            response.body(Body::from(data)).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Read error: {}", e)))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn stream_default_handler(
    Path(id): Path<String>,
    State(state): State<StreamState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    stream_handler(Path((id, 0)), State(state), headers).await
}

/// Media info endpoint - returns codec info and recommended streaming method
async fn media_info_handler(
    Path((id, file_idx)): Path<(String, usize)>,
    State((state, has_ffmpeg)): State<StreamState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"error":"Torrent not found"}"#))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    let file = match session.metainfo.files.get(file_idx) {
        Some(f) => f,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"error":"File not found"}"#))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    let file_path = file.path.to_string_lossy().to_lowercase();
    let extension = file.path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Determine if format is natively supported by browsers
    let (native_supported, codec_type) = match extension.as_str() {
        "mp4" | "m4v" => (true, "video/mp4"),
        "webm" => (true, "video/webm"),
        "mp3" => (true, "audio/mp3"),
        "ogg" | "oga" => (true, "audio/ogg"),
        "wav" => (true, "audio/wav"),
        "mkv" => (false, "video/x-matroska"),
        "avi" => (false, "video/x-msvideo"),
        "mov" => (false, "video/quicktime"),
        "wmv" => (false, "video/x-ms-wmv"),
        "flv" => (false, "video/x-flv"),
        "ts" | "m2ts" => (false, "video/mp2t"),
        "flac" => (false, "audio/flac"),
        _ => (false, "application/octet-stream"),
    };

    // Build response
    let recommend_transcode = !native_supported && has_ffmpeg;
    let streaming_url = if recommend_transcode {
        format!("http://127.0.0.1:3000/transcode/{}/{}/master.m3u8", id, file_idx)
    } else {
        format!("http://127.0.0.1:3000/stream/{}/{}", id, file_idx)
    };

    let json = serde_json::json!({
        "file_name": file.path.file_name().unwrap_or_default().to_string_lossy(),
        "file_size": file.length,
        "mime_type": codec_type,
        "extension": extension,
        "native_supported": native_supported,
        "transcode_available": has_ffmpeg,
        "recommend_transcode": recommend_transcode,
        "streaming_url": streaming_url,
        "direct_url": format!("http://127.0.0.1:3000/stream/{}/{}", id, file_idx),
        "transcode_url": if has_ffmpeg { 
            Some(format!("http://127.0.0.1:3000/transcode/{}/{}/master.m3u8", id, file_idx))
        } else { 
            None 
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(json.to_string()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// HLS master playlist handler
async fn hls_master_handler(
    Path((id, file_idx)): Path<(String, usize)>,
    State((state, has_ffmpeg)): State<StreamState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if !has_ffmpeg {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("FFmpeg not available for transcoding"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Torrent not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    if session.metainfo.files.get(file_idx).is_none() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not found"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    // Return HLS master playlist pointing to adaptive streams
    let playlist = format!(
        "#EXTM3U\n\
         #EXT-X-VERSION:3\n\
         #EXT-X-STREAM-INF:BANDWIDTH=2000000,RESOLUTION=1920x1080\n\
         stream.m3u8\n"
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(playlist))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// HLS playlist handler - generates playlist for on-the-fly transcoding
async fn hls_playlist_handler(
    Path((id, file_idx)): Path<(String, usize)>,
    State((state, _has_ffmpeg)): State<StreamState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Torrent not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    let file = match session.metainfo.files.get(file_idx) {
        Some(f) => f,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    // Estimate duration (assume ~10MB/s for video, 10 second segments)
    let estimated_duration = (file.length as f64 / (5_000_000.0)) as u64; // rough estimate
    let segment_duration = 10;
    let num_segments = (estimated_duration / segment_duration).max(1);

    let mut playlist = String::from("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:10\n#EXT-X-MEDIA-SEQUENCE:0\n");

    for i in 0..num_segments {
        playlist.push_str(&format!("#EXTINF:{}.0,\nsegment{}.ts\n", segment_duration, i));
    }
    playlist.push_str("#EXT-X-ENDLIST\n");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(playlist))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// HLS segment handler - transcodes segments on demand
async fn hls_segment_handler(
    Path((id, file_idx, seg)): Path<(String, usize, String)>,
    State((state, has_ffmpeg)): State<StreamState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if !has_ffmpeg {
        return Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("FFmpeg not available"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let session = match state.sessions.get(&id) {
        Some(s) => s,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Torrent not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    let file = match session.metainfo.files.get(file_idx) {
        Some(f) => f,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    // Parse segment number (e.g., "segment0" -> 0)
    let segment_num: u64 = seg.trim_start_matches("segment")
        .trim_end_matches(".ts")
        .parse()
        .unwrap_or(0);

    let segment_duration = 10u64;
    let start_time = segment_num * segment_duration;

    // Get the file path on disk
    let full_path = session.download_path.join(&session.metainfo.name).join(&file.path);

    if !full_path.exists() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not yet downloaded"))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    // Transcode segment using FFmpeg
    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-ss", &start_time.to_string(),
            "-i", full_path.to_str().unwrap_or(""),
            "-t", &segment_duration.to_string(),
            "-c:v", "libx264",
            "-preset", "ultrafast",
            "-tune", "zerolatency",
            "-crf", "23",
            "-c:a", "aac",
            "-b:a", "128k",
            "-f", "mpegts",
            "-movflags", "+faststart",
            "-"
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "video/mp2t")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(result.stdout))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
        Ok(_) => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Transcoding failed"))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
        Err(e) => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("FFmpeg error: {}", e)))
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
    // Early return for zero-length files - no valid range possible
    if file_size == 0 {
        return None;
    }

    let range_str = range_str.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    // file_size > 0 guaranteed at this point
    let last_byte = file_size - 1;

    let start: u64 = if parts[0].is_empty() {
        let suffix: u64 = parts[1].parse().ok()?;
        file_size.saturating_sub(suffix)
    } else {
        parts[0].parse().ok()?
    };

    let end: u64 = if parts[1].is_empty() {
        last_byte
    } else {
        parts[1].parse().ok()?
    };

    if start > end || start >= file_size {
        return None;
    }

    Some((start, std::cmp::min(end, last_byte)))
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn validate_torrent_id(id: &str) -> Result<(), &'static str> {
    if id.len() != 40 {
        return Err("Invalid torrent ID: must be 40 hex characters");
    }
    if !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid torrent ID: must contain only hex characters");
    }
    Ok(())
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let data = input
        .find(',')
        .map(|idx| &input[idx + 1..])
        .unwrap_or(input);

    BASE64
        .decode(data)
        .map_err(|e| format!("Invalid base64: {}", e))
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    format!("{:x}", now)
}

/// Run the engine standalone (for testing)
pub async fn run() -> Result<()> {
    let engine = Engine::new().await?;
    shutdown_signal().await;
    engine.shutdown().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down..."),
        _ = terminate => info!("Received SIGTERM, shutting down..."),
    }
}
