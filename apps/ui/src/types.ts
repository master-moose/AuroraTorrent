// =============================================================================
// File Priority
// =============================================================================

export type FilePriority = 'Skip' | 'Low' | 'Normal' | 'High';

// =============================================================================
// File Info
// =============================================================================

export interface FileInfo {
    index: number;
    name: string;
    path: string;
    size: number;
    progress: number;
    priority: FilePriority;
    selected: boolean;
}

// =============================================================================
// Peer Info
// =============================================================================

export interface PeerInfo {
    ip: string;
    port: number;
    client: string;
    down_speed: number;
    up_speed: number;
    progress: number;
    flags: string;
    connection_type: string;
    country?: string;
    country_code?: string;
}

// =============================================================================
// Tracker Info
// =============================================================================

export type TrackerStatus = 'Working' | 'Updating' | 'NotWorking' | 'NotContacted' | 'Disabled';

export interface TrackerInfo {
    url: string;
    status: string;
    message?: string;
    peers: number;
    seeds: number;
    leechers: number;
    downloaded: number;
    tier: number;
    next_announce?: number;
}

// =============================================================================
// Category
// =============================================================================

export interface Category {
    name: string;
    save_path?: string;
}

// =============================================================================
// Share Limit Action
// =============================================================================

export type ShareLimitAction = 'Stop' | 'Remove' | 'RemoveWithContent' | 'EnableSuperSeeding' | 'Nothing';

// =============================================================================
// Torrent Limits
// =============================================================================

export interface TorrentLimits {
    max_download_speed?: number;
    max_upload_speed?: number;
    max_connections?: number;
    max_uploads?: number;
    share_ratio_limit?: number;
    seeding_time_limit?: number;
    share_limit_action: ShareLimitAction;
}

// =============================================================================
// Torrent State
// =============================================================================

export interface Torrent {
    id: string;
    name: string;
    progress: number;
    status: string;
    download_speed: number;
    upload_speed: number;
    downloaded: number;
    uploaded: number;
    total_size: number;
    files: FileInfo[];
    peers: PeerInfo[];
    trackers: TrackerInfo[];
    seeds: number;
    leechers: number;
    queue_position: number;
    eta: number;
    added_on: number;
    completed_on: number;
    category?: string;
    tags: string[];
    save_path: string;
    ratio: number;
    seeding_time: number;
    sequential_download: boolean;
    first_last_piece_priority: boolean;
    auto_managed: boolean;
    super_seeding: boolean;
    force_start: boolean;
    limits: TorrentLimits;
    piece_states: number[];
    comment?: string;
    created_by?: string;
    creation_date?: number;
    is_private: boolean;
    magnet_uri?: string;
    num_pieces: number;
    piece_size: number;
    last_activity: number;
    downloaded_session: number;
    uploaded_session: number;
    amount_left: number;
    wasted: number;
    connected_seeds: number;
    connected_leechers: number;
}

// =============================================================================
// Queue Settings
// =============================================================================

export interface QueueSettings {
    max_active_downloads: number;
    max_active_uploads: number;
    max_active_torrents: number;
    slow_torrent_download_rate: number;
    slow_torrent_upload_rate: number;
    slow_torrent_inactive_time: number;
    ignore_slow_torrents: boolean;
    download_queue_enabled: boolean;
    upload_queue_enabled: boolean;
}

// =============================================================================
// Connection Settings
// =============================================================================

export interface ConnectionSettings {
    listen_port: number;
    upnp_enabled: boolean;
    random_port: boolean;
    max_connections: number;
    max_connections_per_torrent: number;
    max_uploads: number;
    max_uploads_per_torrent: number;
}

// =============================================================================
// Proxy Settings
// =============================================================================

export type ProxyType = 'None' | 'Http' | 'Socks4' | 'Socks5' | 'Socks5WithAuth' | 'HttpWithAuth';

export interface ProxySettings {
    enabled: boolean;
    proxy_type: ProxyType;
    host: string;
    port: number;
    username?: string;
    password?: string;
    use_for_peer_connections: boolean;
    use_for_tracker_connections: boolean;
    use_for_dht: boolean;
}

// =============================================================================
// Encryption Settings
// =============================================================================

export type EncryptionMode = 'Prefer' | 'ForceOn' | 'ForceOff';

// =============================================================================
// BitTorrent Settings
// =============================================================================

export interface BitTorrentSettings {
    dht_enabled: boolean;
    pex_enabled: boolean;
    lsd_enabled: boolean;
    encryption: EncryptionMode;
    anonymous_mode: boolean;
    add_trackers_enabled: boolean;
    additional_trackers: string[];
    global_share_ratio_limit?: number;
    global_seeding_time_limit?: number;
    share_limit_action: ShareLimitAction;
}

// =============================================================================
// IP Filter Settings
// =============================================================================

export interface IpFilterSettings {
    enabled: boolean;
    filter_path?: string;
    banned_ips: string[];
}

// =============================================================================
// Bandwidth Scheduler
// =============================================================================

export type ScheduleDays = 'EveryDay' | 'Weekdays' | 'Weekends' | 'Monday' | 'Tuesday' | 'Wednesday' | 'Thursday' | 'Friday' | 'Saturday' | 'Sunday';

export interface ScheduleEntry {
    days: ScheduleDays;
    start_hour: number;
    start_minute: number;
    end_hour: number;
    end_minute: number;
    use_alt_speed: boolean;
}

export interface BandwidthScheduler {
    enabled: boolean;
    schedule: ScheduleEntry[];
    alt_download_limit: number;
    alt_upload_limit: number;
}

// =============================================================================
// Watched Folder
// =============================================================================

export type WatchedFolderAction = 'MonitorAndAddToDefault' | 'MonitorAndAddToCategory';
export type ContentLayout = 'Original' | 'CreateSubfolder' | 'NoSubfolder';

export interface WatchedFolder {
    path: string;
    action: WatchedFolderAction;
    category?: string;
    add_paused: boolean;
    skip_checking: boolean;
    content_layout: ContentLayout;
}

// =============================================================================
// Auto-run Settings
// =============================================================================

export interface AutoRunSettings {
    enabled_on_added: boolean;
    program_on_added: string;
    enabled_on_finished: boolean;
    program_on_finished: string;
}

// =============================================================================
// WebUI Settings
// =============================================================================

export interface WebUISettings {
    enabled: boolean;
    address: string;
    port: number;
    use_upnp: boolean;
    username: string;
    password_hash: string;
    https_enabled: boolean;
    https_cert_path?: string;
    https_key_path?: string;
    localhost_auth_bypass: boolean;
    clickjacking_protection: boolean;
    csrf_protection: boolean;
    host_header_validation: boolean;
}

// =============================================================================
// Complete Config
// =============================================================================

export interface Config {
    // Downloads
    download_path: string;
    temp_path?: string;
    use_temp_path: boolean;
    preallocate_all: boolean;
    incomplete_extension: boolean;
    content_layout: ContentLayout;

    // Speed limits
    max_download_speed: number;
    max_upload_speed: number;
    alt_download_speed: number;
    alt_upload_speed: number;
    use_alt_speed_limits: boolean;

    // Queue
    queue: QueueSettings;

    // Connection
    connection: ConnectionSettings;

    // Proxy
    proxy: ProxySettings;

    // BitTorrent
    bittorrent: BitTorrentSettings;

    // IP Filter
    ip_filter: IpFilterSettings;

    // Scheduler
    scheduler: BandwidthScheduler;

    // Categories
    categories: Record<string, Category>;

    // Tags
    tags: string[];

    // Watched folders
    watched_folders: WatchedFolder[];

    // Auto-run
    auto_run: AutoRunSettings;

    // WebUI
    webui: WebUISettings;

    // Behavior
    auto_start: boolean;
    start_on_launch: boolean;
    confirm_delete: boolean;
    confirm_remove_tracker: boolean;
    delete_torrent_files: boolean;
    show_notifications: boolean;

    // UI
    theme: string;
    locale: string;
    speed_in_title: boolean;
    minimize_to_tray: boolean;
    close_to_tray: boolean;
    start_minimized: boolean;

    // Power Management
    prevent_sleep_downloading: boolean;
    prevent_sleep_seeding: boolean;

    // Actions
    action_on_completion: string;

    // RSS
    rss_refresh_interval: number;
    rss_max_articles_per_feed: number;
    rss_auto_download_enabled: boolean;

    // Search
    search_enabled: boolean;
    search_history_length: number;
}

// =============================================================================
// RSS Types
// =============================================================================

export interface RssFeed {
    id: string;
    name: string;
    url: string;
    enabled: boolean;
    refresh_interval: number;
    last_refresh?: number;
    auto_download: boolean;
}

export interface RssDownloadRule {
    id: string;
    name: string;
    enabled: boolean;
    must_contain: string;
    must_not_contain: string;
    use_regex: boolean;
    episode_filter?: string;
    smart_filter: boolean;
    affected_feeds: string[];
    save_path?: string;
    category?: string;
    add_paused?: boolean;
    assigned_category?: string;
}

export interface RssArticle {
    id: string;
    feed_id: string;
    title: string;
    url: string;
    torrent_url?: string;
    description?: string;
    date?: number;
    is_read: boolean;
    is_downloaded: boolean;
}

// =============================================================================
// Search Types
// =============================================================================

export interface SearchPlugin {
    name: string;
    url: string;
    enabled: boolean;
    supported_categories: string[];
    version: string;
}

export interface SearchResult {
    name: string;
    size: number;
    seeds: number;
    leechers: number;
    engine: string;
    download_url: string;
    description_url?: string;
}

// =============================================================================
// Session Statistics
// =============================================================================

export interface SessionStats {
    total_downloaded: number;
    total_uploaded: number;
    total_wasted: number;
    total_downloaded_session: number;
    total_uploaded_session: number;
    total_torrents: number;
    downloading_torrents: number;
    seeding_torrents: number;
    paused_torrents: number;
    checking_torrents: number;
    error_torrents: number;
    global_ratio: number;
    dht_nodes: number;
    peers_connected: number;
    download_rate: number;
    upload_rate: number;
    disk_read_rate: number;
    disk_write_rate: number;
    disk_cache_size: number;
    disk_cache_usage: number;
    up_time: number;
}

// =============================================================================
// Speed Sample (for graphing)
// =============================================================================

export interface SpeedSample {
    timestamp: number;
    download_rate: number;
    upload_rate: number;
}

// =============================================================================
// Log Types
// =============================================================================

export type LogMessageType = 'Normal' | 'Info' | 'Warning' | 'Critical';

export interface LogMessage {
    id: number;
    timestamp: number;
    message_type: LogMessageType;
    message: string;
}

// =============================================================================
// Notification Types
// =============================================================================

export type NotificationType = 'TorrentAdded' | 'TorrentFinished' | 'TorrentError' | 'ConnectionError' | 'IoError';

export interface Notification {
    id: string;
    notification_type: NotificationType;
    title: string;
    message: string;
    timestamp: number;
    torrent_id?: string;
}

// =============================================================================
// Add Torrent Parameters
// =============================================================================

export interface AddTorrentParams {
    save_path?: string;
    category?: string;
    tags?: string[];
    add_paused?: boolean;
    add_to_top_of_queue?: boolean;
    skip_checking?: boolean;
    content_layout?: ContentLayout;
    sequential_download?: boolean;
    first_last_piece_priority?: boolean;
    download_limit?: number;
    upload_limit?: number;
    rename_to?: string;
    file_priorities?: FilePriority[];
    auto_managed?: boolean;
}
