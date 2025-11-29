//! Configuration management for AuroraTorrent
//!
//! Comprehensive settings matching qBittorrent's feature set

use bridge::{
    AutoRunSettings, BandwidthScheduler, BitTorrentSettings, Category, ConnectionSettings,
    ContentLayout, IpFilterSettings, ProxySettings, QueueSettings, WatchedFolder, WebUISettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // === Downloads ===
    /// Default download path
    pub download_path: String,
    /// Temporary download path (incomplete downloads)
    pub temp_path: Option<String>,
    /// Use temporary path for incomplete downloads
    pub use_temp_path: bool,
    /// Pre-allocate disk space for files
    pub preallocate_all: bool,
    /// Append .!qb extension to incomplete files
    pub incomplete_extension: bool,
    /// Torrent content layout
    pub content_layout: ContentLayout,

    // === Speed limits ===
    /// Global download speed limit (bytes/s, 0 = unlimited)
    pub max_download_speed: u64,
    /// Global upload speed limit (bytes/s, 0 = unlimited)
    pub max_upload_speed: u64,
    /// Alternative download speed limit (for scheduler)
    pub alt_download_speed: u64,
    /// Alternative upload speed limit (for scheduler)
    pub alt_upload_speed: u64,
    /// Use alternative speed limits
    pub use_alt_speed_limits: bool,

    // === Queue ===
    pub queue: QueueSettings,

    // === Connection ===
    pub connection: ConnectionSettings,

    // === Proxy ===
    pub proxy: ProxySettings,

    // === BitTorrent protocol ===
    pub bittorrent: BitTorrentSettings,

    // === IP Filter ===
    pub ip_filter: IpFilterSettings,

    // === Bandwidth Scheduler ===
    pub scheduler: BandwidthScheduler,

    // === Categories ===
    pub categories: HashMap<String, Category>,

    // === Tags ===
    pub tags: Vec<String>,

    // === Watched Folders ===
    pub watched_folders: Vec<WatchedFolder>,

    // === Auto-run ===
    pub auto_run: AutoRunSettings,

    // === WebUI ===
    pub webui: WebUISettings,

    // === Behavior ===
    /// Auto-start torrents when added
    pub auto_start: bool,
    /// Start torrents on application launch
    pub start_on_launch: bool,
    /// Confirm before deleting torrents
    pub confirm_delete: bool,
    /// Confirm before removing trackers
    pub confirm_remove_tracker: bool,
    /// Delete .torrent files after adding
    pub delete_torrent_files: bool,
    /// Show desktop notifications
    pub show_notifications: bool,

    // === UI ===
    /// Theme (dark/light/system)
    pub theme: String,
    /// Locale/language code
    pub locale: String,
    /// Show speed in title bar
    pub speed_in_title: bool,
    /// Minimize to tray
    pub minimize_to_tray: bool,
    /// Close to tray
    pub close_to_tray: bool,
    /// Start minimized
    pub start_minimized: bool,

    // === Power Management ===
    /// Prevent sleep when downloading
    pub prevent_sleep_downloading: bool,
    /// Prevent sleep when seeding
    pub prevent_sleep_seeding: bool,

    // === Actions on completion ===
    /// Action on all downloads complete (none/exit/shutdown/hibernate/sleep)
    pub action_on_completion: String,

    // === RSS ===
    pub rss_refresh_interval: u32,
    pub rss_max_articles_per_feed: u32,
    pub rss_auto_download_enabled: bool,

    // === Search ===
    pub search_enabled: bool,
    pub search_history_length: u32,
}

impl Default for Config {
    fn default() -> Self {
        // Use user's Downloads folder as default
        let download_path = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("AuroraTorrent")
            .to_string_lossy()
            .to_string();

        Self {
            // Downloads
            download_path,
            temp_path: None,
            use_temp_path: false,
            preallocate_all: false,
            incomplete_extension: false,
            content_layout: ContentLayout::Original,

            // Speed limits
            max_download_speed: 0,
            max_upload_speed: 0,
            alt_download_speed: 1024 * 1024, // 1 MB/s
            alt_upload_speed: 512 * 1024,    // 512 KB/s
            use_alt_speed_limits: false,

            // Queue
            queue: QueueSettings::default(),

            // Connection
            connection: ConnectionSettings::default(),

            // Proxy
            proxy: ProxySettings::default(),

            // BitTorrent
            bittorrent: BitTorrentSettings::default(),

            // IP Filter
            ip_filter: IpFilterSettings::default(),

            // Scheduler
            scheduler: BandwidthScheduler::default(),

            // Categories
            categories: HashMap::new(),

            // Tags
            tags: vec![],

            // Watched folders
            watched_folders: vec![],

            // Auto-run
            auto_run: AutoRunSettings::default(),

            // WebUI
            webui: WebUISettings::default(),

            // Behavior
            auto_start: true,
            start_on_launch: true,
            confirm_delete: true,
            confirm_remove_tracker: true,
            delete_torrent_files: false,
            show_notifications: true,

            // UI
            theme: "dark".to_string(),
            locale: "en".to_string(),
            speed_in_title: true,
            minimize_to_tray: false,
            close_to_tray: false,
            start_minimized: false,

            // Power Management
            prevent_sleep_downloading: true,
            prevent_sleep_seeding: false,

            // Actions
            action_on_completion: "none".to_string(),

            // RSS
            rss_refresh_interval: 30, // minutes
            rss_max_articles_per_feed: 50,
            rss_auto_download_enabled: true,

            // Search
            search_enabled: true,
            search_history_length: 20,
        }
    }
}

impl Config {
    /// Get the effective download speed limit based on scheduler
    pub fn effective_download_limit(&self) -> u64 {
        if self.use_alt_speed_limits {
            self.alt_download_speed
        } else {
            self.max_download_speed
        }
    }

    /// Get the effective upload speed limit based on scheduler
    pub fn effective_upload_limit(&self) -> u64 {
        if self.use_alt_speed_limits {
            self.alt_upload_speed
        } else {
            self.max_upload_speed
        }
    }

    /// Add a new category
    pub fn add_category(&mut self, name: String, save_path: Option<String>) {
        self.categories.insert(
            name.clone(),
            Category {
                name,
                save_path,
            },
        );
    }

    /// Remove a category
    pub fn remove_category(&mut self, name: &str) -> bool {
        self.categories.remove(name).is_some()
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    /// Add a watched folder
    pub fn add_watched_folder(&mut self, folder: WatchedFolder) {
        // Don't add duplicates
        if !self.watched_folders.iter().any(|f| f.path == folder.path) {
            self.watched_folders.push(folder);
        }
    }

    /// Remove a watched folder
    pub fn remove_watched_folder(&mut self, path: &str) -> bool {
        if let Some(pos) = self.watched_folders.iter().position(|f| f.path == path) {
            self.watched_folders.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if an IP is banned
    pub fn is_ip_banned(&self, ip: &str) -> bool {
        self.ip_filter.enabled && self.ip_filter.banned_ips.contains(&ip.to_string())
    }

    /// Ban an IP address
    pub fn ban_ip(&mut self, ip: String) {
        if !self.ip_filter.banned_ips.contains(&ip) {
            self.ip_filter.banned_ips.push(ip);
        }
    }

    /// Unban an IP address
    pub fn unban_ip(&mut self, ip: &str) -> bool {
        if let Some(pos) = self.ip_filter.banned_ips.iter().position(|i| i == ip) {
            self.ip_filter.banned_ips.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auto_start);
        assert_eq!(config.max_download_speed, 0);
        assert_eq!(config.queue.max_active_downloads, 3);
    }

    #[test]
    fn test_categories() {
        let mut config = Config::default();
        config.add_category("Movies".to_string(), Some("/movies".to_string()));
        assert!(config.categories.contains_key("Movies"));
        assert!(config.remove_category("Movies"));
        assert!(!config.categories.contains_key("Movies"));
    }

    #[test]
    fn test_tags() {
        let mut config = Config::default();
        config.add_tag("favorite".to_string());
        assert!(config.tags.contains(&"favorite".to_string()));
        assert!(config.remove_tag("favorite"));
        assert!(!config.tags.contains(&"favorite".to_string()));
    }

    #[test]
    fn test_ip_filter() {
        let mut config = Config::default();
        config.ip_filter.enabled = true;
        config.ban_ip("192.168.1.100".to_string());
        assert!(config.is_ip_banned("192.168.1.100"));
        assert!(!config.is_ip_banned("192.168.1.101"));
        assert!(config.unban_ip("192.168.1.100"));
        assert!(!config.is_ip_banned("192.168.1.100"));
    }
}
