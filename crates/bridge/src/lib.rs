//! Bridge types shared between the engine and UI
//!
//! These types are used for direct Tauri IPC communication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File download priority
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FilePriority {
    Skip = 0,
    Low = 1,
    Normal = 2,
    High = 3,
}

impl Default for FilePriority {
    fn default() -> Self {
        FilePriority::Normal
    }
}

/// Information about a file within a torrent
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub index: usize,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub progress: f64,
    pub priority: FilePriority,
    pub selected: bool,
}

/// Information about a connected peer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
    pub client: String,
    pub down_speed: u64,
    pub up_speed: u64,
    pub progress: f64,
    pub flags: String,
    pub connection_type: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
}

impl Default for PeerInfo {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: 0,
            client: String::new(),
            down_speed: 0,
            up_speed: 0,
            progress: 0.0,
            flags: String::new(),
            connection_type: "TCP".to_string(),
            country: None,
            country_code: None,
        }
    }
}

/// Tracker status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum TrackerStatus {
    Working,
    Updating,
    NotWorking,
    NotContacted,
    Disabled,
}

impl Default for TrackerStatus {
    fn default() -> Self {
        TrackerStatus::NotContacted
    }
}

/// Information about a tracker
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackerInfo {
    pub url: String,
    pub status: String,
    pub message: Option<String>,
    pub peers: u32,
    pub seeds: u32,
    pub leechers: u32,
    pub downloaded: u32,
    pub tier: u32,
    pub next_announce: Option<u64>,
}

impl Default for TrackerInfo {
    fn default() -> Self {
        Self {
            url: String::new(),
            status: "Not contacted".to_string(),
            message: None,
            peers: 0,
            seeds: 0,
            leechers: 0,
            downloaded: 0,
            tier: 0,
            next_announce: None,
        }
    }
}

/// Torrent category with save path options
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub name: String,
    pub save_path: Option<String>,
}

/// Share limit action
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ShareLimitAction {
    Stop,
    Remove,
    RemoveWithContent,
    EnableSuperSeeding,
    Nothing,
}

impl Default for ShareLimitAction {
    fn default() -> Self {
        ShareLimitAction::Nothing
    }
}

/// Per-torrent limits
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TorrentLimits {
    pub max_download_speed: Option<u64>,
    pub max_upload_speed: Option<u64>,
    pub max_connections: Option<u32>,
    pub max_uploads: Option<u32>,
    pub share_ratio_limit: Option<f64>,
    pub seeding_time_limit: Option<u64>,
    pub share_limit_action: ShareLimitAction,
}

/// Current state of a torrent
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorrentState {
    pub id: String,
    pub name: String,
    pub progress: f64,
    /// Status: "Queued", "Starting", "Downloading", "Seeding", "Paused", "Stopped", "Error", "Checking", "FetchingMetadata", "ForcedDownloading", "ForcedSeeding", "Moving"
    pub status: String,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub total_size: u64,
    pub files: Vec<FileInfo>,
    pub peers: Vec<PeerInfo>,
    pub trackers: Vec<TrackerInfo>,
    pub seeds: u32,
    pub leechers: u32,
    /// Position in download queue (0 = downloading now)
    pub queue_position: u32,
    /// ETA in seconds (0 = unknown)
    pub eta: u64,
    /// Added timestamp
    pub added_on: u64,
    /// Completion timestamp (0 if not complete)
    pub completed_on: u64,
    /// Category name
    pub category: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Save path
    pub save_path: String,
    /// Share ratio (uploaded / downloaded)
    pub ratio: f64,
    /// Total seeding time in seconds
    pub seeding_time: u64,
    /// Is sequential download enabled
    pub sequential_download: bool,
    /// First/last piece priority
    pub first_last_piece_priority: bool,
    /// Auto-managed by queue
    pub auto_managed: bool,
    /// Super seeding mode
    pub super_seeding: bool,
    /// Force start (ignore queue)
    pub force_start: bool,
    /// Per-torrent limits
    pub limits: TorrentLimits,
    /// Piece states for visualization (0=missing, 1=downloading, 2=have)
    pub piece_states: Vec<u8>,
    /// Comment from torrent file
    pub comment: Option<String>,
    /// Created by field from torrent
    pub created_by: Option<String>,
    /// Creation date from torrent
    pub creation_date: Option<u64>,
    /// Private torrent flag
    pub is_private: bool,
    /// Magnet URI
    pub magnet_uri: Option<String>,
    /// Number of pieces
    pub num_pieces: u32,
    /// Piece size
    pub piece_size: u64,
    /// Time since last activity
    pub last_activity: u64,
    /// Downloaded this session
    pub downloaded_session: u64,
    /// Uploaded this session
    pub uploaded_session: u64,
    /// Amount of data left to download
    pub amount_left: u64,
    /// Wasted data (hash failures)
    pub wasted: u64,
    /// Connected seeds count
    pub connected_seeds: u32,
    /// Connected leechers count
    pub connected_leechers: u32,
}

impl Default for TorrentState {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            progress: 0.0,
            status: "Stopped".to_string(),
            download_speed: 0,
            upload_speed: 0,
            downloaded: 0,
            uploaded: 0,
            total_size: 0,
            files: vec![],
            peers: vec![],
            trackers: vec![],
            seeds: 0,
            leechers: 0,
            queue_position: 0,
            eta: 0,
            added_on: 0,
            completed_on: 0,
            category: None,
            tags: vec![],
            save_path: String::new(),
            ratio: 0.0,
            seeding_time: 0,
            sequential_download: false,
            first_last_piece_priority: false,
            auto_managed: true,
            super_seeding: false,
            force_start: false,
            limits: TorrentLimits::default(),
            piece_states: vec![],
            comment: None,
            created_by: None,
            creation_date: None,
            is_private: false,
            magnet_uri: None,
            num_pieces: 0,
            piece_size: 0,
            last_activity: 0,
            downloaded_session: 0,
            uploaded_session: 0,
            amount_left: 0,
            wasted: 0,
            connected_seeds: 0,
            connected_leechers: 0,
        }
    }
}

/// Queue settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueueSettings {
    pub max_active_downloads: u32,
    pub max_active_uploads: u32,
    pub max_active_torrents: u32,
    pub slow_torrent_download_rate: u32,
    pub slow_torrent_upload_rate: u32,
    pub slow_torrent_inactive_time: u32,
    pub ignore_slow_torrents: bool,
    pub download_queue_enabled: bool,
    pub upload_queue_enabled: bool,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            max_active_downloads: 3,
            max_active_uploads: 5,
            max_active_torrents: 5,
            slow_torrent_download_rate: 2, // KB/s
            slow_torrent_upload_rate: 2,   // KB/s
            slow_torrent_inactive_time: 60, // seconds
            ignore_slow_torrents: false,
            download_queue_enabled: true,
            upload_queue_enabled: true,
        }
    }
}

/// Connection settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionSettings {
    pub listen_port: u16,
    pub upnp_enabled: bool,
    pub random_port: bool,
    pub max_connections: u32,
    pub max_connections_per_torrent: u32,
    pub max_uploads: u32,
    pub max_uploads_per_torrent: u32,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            listen_port: 6881,
            upnp_enabled: true,
            random_port: false,
            max_connections: 500,
            max_connections_per_torrent: 100,
            max_uploads: 20,
            max_uploads_per_torrent: 4,
        }
    }
}

/// Proxy type
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    None,
    Http,
    Socks4,
    Socks5,
    Socks5WithAuth,
    HttpWithAuth,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::None
    }
}

/// Proxy settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProxySettings {
    pub enabled: bool,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_for_peer_connections: bool,
    pub use_for_tracker_connections: bool,
    pub use_for_dht: bool,
}

/// Encryption mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMode {
    Prefer,
    ForceOn,
    ForceOff,
}

impl Default for EncryptionMode {
    fn default() -> Self {
        EncryptionMode::Prefer
    }
}

/// BitTorrent protocol settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitTorrentSettings {
    pub dht_enabled: bool,
    pub pex_enabled: bool,
    pub lsd_enabled: bool,
    pub encryption: EncryptionMode,
    pub anonymous_mode: bool,
    pub add_trackers_enabled: bool,
    pub additional_trackers: Vec<String>,
    pub global_share_ratio_limit: Option<f64>,
    pub global_seeding_time_limit: Option<u64>,
    pub share_limit_action: ShareLimitAction,
}

impl Default for BitTorrentSettings {
    fn default() -> Self {
        Self {
            dht_enabled: true,
            pex_enabled: true,
            lsd_enabled: true,
            encryption: EncryptionMode::Prefer,
            anonymous_mode: false,
            add_trackers_enabled: false,
            additional_trackers: vec![],
            global_share_ratio_limit: None,
            global_seeding_time_limit: None,
            share_limit_action: ShareLimitAction::Nothing,
        }
    }
}

/// Schedule day selection
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDays {
    EveryDay,
    Weekdays,
    Weekends,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Default for ScheduleDays {
    fn default() -> Self {
        ScheduleDays::EveryDay
    }
}

/// Bandwidth scheduler settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BandwidthScheduler {
    pub enabled: bool,
    pub schedule: Vec<ScheduleEntry>,
    pub alt_download_limit: u64,
    pub alt_upload_limit: u64,
}

impl Default for BandwidthScheduler {
    fn default() -> Self {
        Self {
            enabled: false,
            schedule: vec![],
            alt_download_limit: 0,
            alt_upload_limit: 0,
        }
    }
}

/// Schedule entry for bandwidth scheduling
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleEntry {
    pub days: ScheduleDays,
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
    pub use_alt_speed: bool,
}

/// IP filter settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct IpFilterSettings {
    pub enabled: bool,
    pub filter_path: Option<String>,
    pub banned_ips: Vec<String>,
}

/// Watched folder action
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WatchedFolderAction {
    MonitorAndAddToDefault,
    MonitorAndAddToCategory,
}

impl Default for WatchedFolderAction {
    fn default() -> Self {
        WatchedFolderAction::MonitorAndAddToDefault
    }
}

/// Watched folder configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchedFolder {
    pub path: String,
    pub action: WatchedFolderAction,
    pub category: Option<String>,
    pub add_paused: bool,
    pub skip_checking: bool,
    pub content_layout: ContentLayout,
}

/// Torrent content layout
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ContentLayout {
    Original,
    CreateSubfolder,
    NoSubfolder,
}

impl Default for ContentLayout {
    fn default() -> Self {
        ContentLayout::Original
    }
}

/// Auto-run script settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AutoRunSettings {
    pub enabled_on_added: bool,
    pub program_on_added: String,
    pub enabled_on_finished: bool,
    pub program_on_finished: String,
}

/// Web UI settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebUISettings {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
    pub use_upnp: bool,
    pub username: String,
    pub password_hash: String,
    pub https_enabled: bool,
    pub https_cert_path: Option<String>,
    pub https_key_path: Option<String>,
    pub localhost_auth_bypass: bool,
    pub clickjacking_protection: bool,
    pub csrf_protection: bool,
    pub host_header_validation: bool,
}

impl Default for WebUISettings {
    fn default() -> Self {
        Self {
            enabled: false,
            address: "0.0.0.0".to_string(),
            port: 8080,
            use_upnp: false,
            username: "admin".to_string(),
            password_hash: String::new(),
            https_enabled: false,
            https_cert_path: None,
            https_key_path: None,
            localhost_auth_bypass: true,
            clickjacking_protection: true,
            csrf_protection: true,
            host_header_validation: true,
        }
    }
}

/// RSS feed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RssFeed {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub refresh_interval: u32, // minutes
    pub last_refresh: Option<u64>,
    pub auto_download: bool,
}

/// RSS auto-download rule
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RssDownloadRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub must_contain: String,
    pub must_not_contain: String,
    pub use_regex: bool,
    pub episode_filter: Option<String>,
    pub smart_filter: bool,
    pub affected_feeds: Vec<String>,
    pub save_path: Option<String>,
    pub category: Option<String>,
    pub add_paused: Option<bool>,
    pub assigned_category: Option<String>,
}

/// RSS article
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RssArticle {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub torrent_url: Option<String>,
    pub description: Option<String>,
    pub date: Option<u64>,
    pub is_read: bool,
    pub is_downloaded: bool,
}

/// Search plugin info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchPlugin {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub supported_categories: Vec<String>,
    pub version: String,
}

/// Search result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub size: u64,
    pub seeds: u32,
    pub leechers: u32,
    pub engine: String,
    pub download_url: String,
    pub description_url: Option<String>,
}

/// Session statistics
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SessionStats {
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub total_wasted: u64,
    pub total_downloaded_session: u64,
    pub total_uploaded_session: u64,
    pub total_torrents: u32,
    pub downloading_torrents: u32,
    pub seeding_torrents: u32,
    pub paused_torrents: u32,
    pub checking_torrents: u32,
    pub error_torrents: u32,
    pub global_ratio: f64,
    pub dht_nodes: u32,
    pub peers_connected: u32,
    pub download_rate: u64,
    pub upload_rate: u64,
    pub disk_read_rate: u64,
    pub disk_write_rate: u64,
    pub disk_cache_size: u64,
    pub disk_cache_usage: f64,
    pub up_time: u64, // seconds
}

/// Speed sample for graphing
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct SpeedSample {
    pub timestamp: u64,
    pub download_rate: u64,
    pub upload_rate: u64,
}

/// Log message type
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum LogMessageType {
    Normal,
    Info,
    Warning,
    Critical,
}

/// Log message
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogMessage {
    pub id: u64,
    pub timestamp: u64,
    pub message_type: LogMessageType,
    pub message: String,
}

/// Notification type
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    TorrentAdded,
    TorrentFinished,
    TorrentError,
    ConnectionError,
    IoError,
}

/// Notification
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub torrent_id: Option<String>,
}

/// Add torrent parameters
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AddTorrentParams {
    pub save_path: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub add_paused: Option<bool>,
    pub add_to_top_of_queue: Option<bool>,
    pub skip_checking: Option<bool>,
    pub content_layout: Option<ContentLayout>,
    pub sequential_download: Option<bool>,
    pub first_last_piece_priority: Option<bool>,
    pub download_limit: Option<u64>,
    pub upload_limit: Option<u64>,
    pub rename_to: Option<String>,
    pub file_priorities: Option<Vec<FilePriority>>,
    /// Automatically managed by queue
    pub auto_managed: Option<bool>,
}
