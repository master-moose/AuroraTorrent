//! Torrent Session - Coordinates downloading a single torrent
//!
//! Manages peer connections, piece downloads, and tracker communication

use crate::peer::{generate_peer_id, PeerConnection, PeerMessage};
use crate::piece::PieceManager;
use crate::torrent::TorrentMetainfo;
use crate::tracker::{announce, AnnounceParams, TrackerEvent};
use bytes::Bytes;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::piece::BlockInfo;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex, RwLock, Semaphore};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Maximum concurrent peer connections
const MAX_PEERS: usize = 50;

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Downloading,
    Seeding,
    Paused,
    Stopped,
    Error,
}

/// Statistics for the session
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub downloaded: u64,
    pub uploaded: u64,
    pub download_rate: u64,
    pub upload_rate: u64,
    pub connected_peers: usize,
    pub available_peers: usize,
    pub progress: f64,
    pub seeders: u32,
    pub leechers: u32,
}

/// Active peer info
#[derive(Debug, Clone)]
pub struct PeerStats {
    pub addr: SocketAddr,
    pub client: String,
    pub download_rate: u64,
    pub upload_rate: u64,
}

/// A running torrent session
pub struct TorrentSession {
    /// Torrent metadata
    pub metainfo: Arc<TorrentMetainfo>,
    /// Our peer ID
    peer_id: [u8; 20],
    /// Piece manager
    pieces: Arc<RwLock<PieceManager>>,
    /// Current state
    state: Arc<RwLock<SessionState>>,
    /// Statistics
    stats: Arc<RwLock<SessionStats>>,
    /// Connected peer addresses
    connected_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    /// Peer statistics
    peer_stats: Arc<RwLock<HashMap<SocketAddr, PeerStats>>>,
    /// Available peers from tracker/DHT
    available_peers: Arc<Mutex<VecDeque<SocketAddr>>>,
    /// Download path (stored for resume capability)
    pub download_path: PathBuf,
    /// Whether to use sequential mode
    sequential: AtomicBool,
    /// Stop signal
    stop_tx: broadcast::Sender<()>,
    /// Total downloaded bytes
    total_downloaded: AtomicU64,
    /// Total uploaded bytes
    total_uploaded: AtomicU64,
    /// Listen port for incoming connections
    pub port: u16,
    /// Additional trackers added by user
    extra_trackers: Arc<RwLock<Vec<String>>>,
}

impl TorrentSession {
    /// Create a new session from torrent metadata
    pub fn new(metainfo: TorrentMetainfo, download_path: PathBuf, listen_port: u16) -> Self {
        let peer_id = generate_peer_id();

        let files: Vec<_> = metainfo
            .files
            .iter()
            .map(|f| (f.path.clone(), f.length, f.offset))
            .collect();

        let pieces = PieceManager::new(
            metainfo.pieces.clone(),
            metainfo.piece_length,
            metainfo.total_size,
            files,
            download_path.clone(),
            metainfo.name.clone(),
        );

        let (stop_tx, _) = broadcast::channel(1);

        Self {
            metainfo: Arc::new(metainfo),
            peer_id,
            pieces: Arc::new(RwLock::new(pieces)),
            state: Arc::new(RwLock::new(SessionState::Starting)),
            stats: Arc::new(RwLock::new(SessionStats::default())),
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            peer_stats: Arc::new(RwLock::new(HashMap::new())),
            available_peers: Arc::new(Mutex::new(VecDeque::new())),
            download_path,
            sequential: AtomicBool::new(false),
            stop_tx,
            total_downloaded: AtomicU64::new(0),
            total_uploaded: AtomicU64::new(0),
            port: listen_port,
            extra_trackers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Enable sequential downloading for streaming
    pub async fn set_sequential(&self, enabled: bool) {
        self.sequential.store(enabled, Ordering::Relaxed);
        self.pieces.write().await.set_sequential(enabled);
    }

    /// Set priority pieces for streaming
    pub async fn prioritize_pieces(&self, pieces: Vec<u32>) {
        self.pieces.read().await.set_priority_pieces(pieces);
    }

    /// Start the session
    pub async fn start(self: Arc<Self>) {
        *self.state.write().await = SessionState::Downloading;

        // Start tracker announcer
        let tracker_session = self.clone();
        tokio::spawn(async move {
            tracker_session.tracker_loop().await;
        });

        // Start peer connector
        let connector_session = self.clone();
        tokio::spawn(async move {
            connector_session.peer_connector_loop().await;
        });

        // Start incoming connection listener
        let listener_session = self.clone();
        tokio::spawn(async move {
            listener_session.listen_for_peers().await;
        });

        // Start stats updater
        let stats_session = self.clone();
        tokio::spawn(async move {
            stats_session.stats_update_loop().await;
        });

        info!("Session started for: {}", self.metainfo.name);
    }

    /// Stop the session
    pub async fn stop(&self) {
        *self.state.write().await = SessionState::Stopped;
        let _ = self.stop_tx.send(());
    }

    /// Pause the session
    pub async fn pause(&self) {
        *self.state.write().await = SessionState::Paused;
    }

    /// Resume the session
    pub async fn resume(&self) {
        let is_complete = self.pieces.read().await.is_complete();
        if is_complete {
            *self.state.write().await = SessionState::Seeding;
        } else {
            *self.state.write().await = SessionState::Downloading;
        }
    }

    /// Get current state
    pub async fn state(&self) -> SessionState {
        *self.state.read().await
    }

    /// Get current statistics
    pub async fn stats(&self) -> SessionStats {
        self.stats.read().await.clone()
    }

    /// Get peer statistics
    pub async fn peer_stats(&self) -> Vec<PeerStats> {
        self.peer_stats.read().await.values().cloned().collect()
    }

    /// Get piece manager for streaming
    pub fn piece_manager(&self) -> Arc<RwLock<PieceManager>> {
        self.pieces.clone()
    }

    /// Add peers (e.g., from DHT discovery)
    pub async fn add_peers(&self, peers: Vec<std::net::SocketAddr>) {
        let peer_count = peers.len();
        let connected = self.connected_peers.read().await;
        let mut available = self.available_peers.lock().await;
        for peer in peers {
            if !connected.contains(&peer) && !available.contains(&peer) {
                available.push_back(peer);
            }
        }
        debug!(
            "Added {} DHT peers, total available: {}",
            peer_count,
            available.len()
        );
    }

    /// Tracker announce loop
    async fn tracker_loop(self: Arc<Self>) {
        let mut stop_rx = self.stop_tx.subscribe();
        let mut interval = interval(Duration::from_secs(30));
        let mut first = true;

        loop {
            tokio::select! {
                _ = stop_rx.recv() => break,
                _ = interval.tick() => {
                    let state = *self.state.read().await;
                    if state == SessionState::Paused {
                        continue;
                    }

                    let event = if first {
                        first = false;
                        TrackerEvent::Started
                    } else {
                        TrackerEvent::None
                    };

                    self.announce_to_trackers(event).await;
                }
            }
        }

        // Send stopped event
        self.announce_to_trackers(TrackerEvent::Stopped).await;
    }

    async fn announce_to_trackers(&self, event: TrackerEvent) {
        let left = {
            let pieces = self.pieces.read().await;
            if pieces.is_complete() {
                0
            } else {
                self.metainfo.total_size
                    - (pieces.verified_count() as u64 * self.metainfo.piece_length)
            }
        };

        let params = AnnounceParams {
            info_hash: self.metainfo.info_hash,
            peer_id: self.peer_id,
            port: self.port,
            uploaded: self.total_uploaded.load(Ordering::Relaxed),
            downloaded: self.total_downloaded.load(Ordering::Relaxed),
            left,
            event,
            compact: true,
            numwant: Some(50),
        };

        // Try announce URL first
        let mut trackers: Vec<String> = Vec::new();
        if let Some(announce_url) = &self.metainfo.announce {
            trackers.push(announce_url.clone());
        }
        for tier in &self.metainfo.announce_list {
            trackers.extend(tier.clone());
        }

        for tracker_url in trackers {
            match announce(&tracker_url, &params).await {
                Ok(response) => {
                    debug!(
                        "Tracker {} returned {} peers",
                        tracker_url,
                        response.peers.len()
                    );

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        if let Some(s) = response.complete {
                            stats.seeders = s;
                        }
                        if let Some(l) = response.incomplete {
                            stats.leechers = l;
                        }
                    }

                    // Add peers
                    {
                        let connected = self.connected_peers.read().await;
                        let mut available = self.available_peers.lock().await;
                        for peer in response.peers {
                            if !connected.contains(&peer) {
                                available.push_back(peer);
                            }
                        }
                    }

                    // Success, don't need to try other trackers
                    break;
                }
                Err(e) => {
                    debug!("Tracker {} error: {}", tracker_url, e);
                }
            }
        }
    }

    /// Peer connection loop
    async fn peer_connector_loop(self: Arc<Self>) {
        let mut stop_rx = self.stop_tx.subscribe();
        let semaphore = Arc::new(Semaphore::new(MAX_PEERS));

        loop {
            tokio::select! {
                _ = stop_rx.recv() => break,
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    let state = *self.state.read().await;
                    if state == SessionState::Paused || state == SessionState::Stopped {
                        continue;
                    }

                    let connected = self.connected_peers.read().await.len();
                    if connected >= MAX_PEERS {
                        continue;
                    }

                    // Get next peer to connect to
                    let peer_addr = self.available_peers.lock().await.pop_front();

                    if let Some(addr) = peer_addr {
                        if self.connected_peers.read().await.contains(&addr) {
                            continue;
                        }

                        let permit = match semaphore.clone().try_acquire_owned() {
                            Ok(p) => p,
                            Err(_) => {
                                self.available_peers.lock().await.push_back(addr);
                                continue;
                            }
                        };

                        let session = self.clone();
                        tokio::spawn(async move {
                            session.handle_peer_connection(addr).await;
                            drop(permit);
                        });
                    }
                }
            }
        }
    }

    /// Listen for incoming peer connections
    async fn listen_for_peers(self: Arc<Self>) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", self.port)).await {
            Ok(l) => l,
            Err(e) => {
                warn!("Failed to bind listener: {}", e);
                return;
            }
        };

        info!("Listening for peers on port {}", self.port);
        let mut stop_rx = self.stop_tx.subscribe();

        loop {
            tokio::select! {
                _ = stop_rx.recv() => break,
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            if self.connected_peers.read().await.len() >= MAX_PEERS {
                                continue;
                            }

                            let session = self.clone();
                            tokio::spawn(async move {
                                session.handle_incoming_peer(stream, addr).await;
                            });
                        }
                        Err(e) => {
                            warn!("Accept error: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Handle incoming peer connection
    async fn handle_incoming_peer(self: Arc<Self>, stream: TcpStream, addr: SocketAddr) {
        match PeerConnection::accept(stream, addr, self.metainfo.info_hash, self.peer_id).await {
            Ok(conn) => {
                self.run_peer_session(conn).await;
            }
            Err(e) => {
                debug!("Incoming handshake failed from {}: {}", addr, e);
            }
        }
    }

    /// Handle outgoing peer connection
    async fn handle_peer_connection(self: Arc<Self>, addr: SocketAddr) {
        match PeerConnection::connect(addr, self.metainfo.info_hash, self.peer_id, 10).await {
            Ok(conn) => {
                self.run_peer_session(conn).await;
            }
            Err(e) => {
                debug!("Connection to {} failed: {}", addr, e);
            }
        }
    }

    /// Run session with a connected peer
    async fn run_peer_session(self: Arc<Self>, mut conn: PeerConnection) {
        let addr = conn.addr;

        // Mark as connected
        self.connected_peers.write().await.insert(addr);
        self.peer_stats.write().await.insert(
            addr,
            PeerStats {
                addr,
                client: conn.client_name(),
                download_rate: 0,
                upload_rate: 0,
            },
        );

        debug!("Connected to peer: {} ({})", addr, conn.client_name());

        // Send our bitfield
        let bitfield = self.pieces.read().await.bitfield();
        if let Err(e) = conn.send(PeerMessage::Bitfield { bitfield }).await {
            debug!("Failed to send bitfield to {}: {}", addr, e);
            self.disconnect_peer(addr).await;
            return;
        }

        // Send interested
        if let Err(e) = conn.send(PeerMessage::Interested).await {
            debug!("Failed to send interested to {}: {}", addr, e);
            self.disconnect_peer(addr).await;
            return;
        }
        conn.state.am_interested = true;

        // Main peer loop
        // pending_requests: blocks we've requested but not yet received
        let mut pending_requests: Vec<BlockInfo> = Vec::new();
        // blocks_to_request: blocks we need to request (from the current piece)
        let mut blocks_to_request: VecDeque<BlockInfo> = VecDeque::new();
        let mut last_activity = Instant::now();
        let mut current_piece: Option<u32> = None;

        // Maximum concurrent requests to keep the pipeline full
        const MAX_PENDING: usize = 5;

        loop {
            let state = *self.state.read().await;
            if state == SessionState::Stopped {
                break;
            }

            // Handle timeout
            if last_activity.elapsed() > Duration::from_secs(300) {
                debug!("Peer {} timed out", addr);
                break;
            }

            // Send requests if we can (before waiting for messages)
            if !conn.state.peer_choking && state == SessionState::Downloading {
                // Fill our request queue if needed
                while blocks_to_request.is_empty() && pending_requests.len() < MAX_PENDING {
                    if current_piece.is_none() {
                        let selected = self.pieces.read().await.select_piece(&conn.bitfield);
                        if let Some(piece) = selected {
                            current_piece = Some(piece);
                            let blocks = self.pieces.read().await.start_piece(piece);
                            blocks_to_request.extend(blocks);
                            debug!("Selected piece {} with {} blocks", piece, blocks_to_request.len());
                        } else {
                            // No pieces available from this peer
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Send requests up to MAX_PENDING
                while pending_requests.len() < MAX_PENDING {
                    if let Some(block) = blocks_to_request.pop_front() {
                        if conn
                            .send(PeerMessage::Request {
                                index: block.piece,
                                begin: block.offset,
                                length: block.length,
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        pending_requests.push(block);
                    } else {
                        break;
                    }
                }
            }

            // Receive message with timeout
            let msg = match conn.recv_timeout(Duration::from_secs(30)).await {
                Ok(msg) => {
                    last_activity = Instant::now();
                    msg
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Send keep-alive
                    if conn.send(PeerMessage::KeepAlive).await.is_err() {
                        break;
                    }
                    continue;
                }
                Err(_) => break,
            };

            conn.handle_message(&msg);

            match msg {
                PeerMessage::KeepAlive => {}
                PeerMessage::Choke => {
                    // Cancel pending requests
                    pending_requests.clear();
                    blocks_to_request.clear();
                    if let Some(piece) = current_piece.take() {
                        self.pieces.read().await.cancel_piece(piece);
                    }
                }
                PeerMessage::Unchoke => {
                    // Can request pieces now - will be handled at top of loop
                    debug!("Peer {} unchoked us", addr);
                }
                PeerMessage::Have { piece_index } => {
                    // Peer got a new piece - already handled in handle_message
                    debug!("Peer {} has piece {}", addr, piece_index);
                }
                PeerMessage::Bitfield { .. } => {
                    // Already handled in handle_message
                }
                PeerMessage::Request {
                    index,
                    begin,
                    length,
                } => {
                    // Peer wants data from us (upload)
                    let can_upload = {
                        let pieces = self.pieces.read().await;
                        state == SessionState::Seeding || pieces.have[index as usize]
                    };

                    if can_upload {
                        let data = {
                            let pieces = self.pieces.read().await;
                            pieces
                                .read_data(
                                    index as u64 * self.metainfo.piece_length + begin as u64,
                                    length as usize,
                                )
                                .await
                        };

                        if let Ok(data) = data {
                            let _ = conn
                                .send(PeerMessage::Piece {
                                    index,
                                    begin,
                                    block: Bytes::from(data),
                                })
                                .await;
                            self.total_uploaded
                                .fetch_add(length as u64, Ordering::Relaxed);
                        }
                    }
                }
                PeerMessage::Piece {
                    index,
                    begin,
                    block,
                } => {
                    // Received a block
                    let block_len = block.len() as u64;
                    self.total_downloaded
                        .fetch_add(block_len, Ordering::Relaxed);

                    // Remove from pending
                    pending_requests.retain(|r| !(r.piece == index && r.offset == begin));

                    // Add to piece manager
                    let complete = self.pieces.read().await.receive_block(index, begin, block);

                    if complete {
                        // Verify and write piece
                        let verify_result = {
                            let pieces = self.pieces.read().await;
                            pieces.verify_and_write_piece(index).await
                        };

                        match verify_result {
                            Ok(true) => {
                                // Mark piece as complete
                                self.pieces.write().await.mark_verified(index as usize);
                                info!("Piece {} verified and complete", index);

                                // Check if torrent is complete
                                if self.pieces.read().await.is_complete() {
                                    *self.state.write().await = SessionState::Seeding;
                                    info!("Download complete: {}", self.metainfo.name);
                                }

                                current_piece = None;
                            }
                            Ok(false) => {
                                // Hash mismatch, will retry
                                warn!("Piece {} hash mismatch, will retry", index);
                                current_piece = None;
                            }
                            Err(e) => {
                                warn!("Failed to write piece {}: {}", index, e);
                                current_piece = None;
                            }
                        }
                    }
                }
                PeerMessage::Cancel { .. } => {
                    // Peer cancelled a request
                }
                _ => {}
            }
        }

        self.disconnect_peer(addr).await;
    }

    async fn disconnect_peer(&self, addr: SocketAddr) {
        self.connected_peers.write().await.remove(&addr);
        self.peer_stats.write().await.remove(&addr);
        debug!("Disconnected from peer: {}", addr);
    }

    /// Stats update loop
    async fn stats_update_loop(self: Arc<Self>) {
        let mut stop_rx = self.stop_tx.subscribe();
        let mut interval = interval(Duration::from_secs(1));
        let mut last_downloaded = 0u64;
        let mut last_uploaded = 0u64;

        loop {
            tokio::select! {
                _ = stop_rx.recv() => break,
                _ = interval.tick() => {
                    let downloaded = self.total_downloaded.load(Ordering::Relaxed);
                    let uploaded = self.total_uploaded.load(Ordering::Relaxed);

                    let download_rate = downloaded.saturating_sub(last_downloaded);
                    let upload_rate = uploaded.saturating_sub(last_uploaded);

                    last_downloaded = downloaded;
                    last_uploaded = uploaded;

                    let progress = self.pieces.read().await.progress();
                    let connected = self.connected_peers.read().await.len();
                    let available = self.available_peers.lock().await.len();

                    let mut stats = self.stats.write().await;
                    stats.downloaded = downloaded;
                    stats.uploaded = uploaded;
                    stats.download_rate = download_rate;
                    stats.upload_rate = upload_rate;
                    stats.connected_peers = connected;
                    stats.available_peers = available;
                    stats.progress = progress;
                }
            }
        }
    }

    /// Force verification of all pieces
    pub async fn force_recheck(&self) {
        info!("Force rechecking torrent: {}", self.metainfo.name);
        
        // Set state to show we're checking
        let was_state = *self.state.read().await;
        *self.state.write().await = SessionState::Starting;
        
        // Verify all pieces
        {
            let mut pieces = self.pieces.write().await;
            pieces.verify_all().await;
        }
        
        // Restore state or set to seeding if complete
        let is_complete = self.pieces.read().await.is_complete();
        if is_complete {
            *self.state.write().await = SessionState::Seeding;
        } else if was_state == SessionState::Paused {
            *self.state.write().await = SessionState::Paused;
        } else {
            *self.state.write().await = SessionState::Downloading;
        }
        
        info!("Recheck complete for: {}", self.metainfo.name);
    }

    /// Force announce to all trackers
    pub async fn force_reannounce(&self) {
        info!("Force reannouncing torrent: {}", self.metainfo.name);
        self.announce_to_trackers(TrackerEvent::None).await;
    }

    /// Add trackers to this torrent
    pub async fn add_trackers(&self, trackers: Vec<String>) {
        let mut extra = self.extra_trackers.write().await;
        for tracker in trackers {
            if !extra.contains(&tracker) {
                info!("Added tracker: {}", tracker);
                extra.push(tracker);
            }
        }
    }

    /// Remove trackers from this torrent
    pub async fn remove_trackers(&self, trackers: Vec<String>) {
        let mut extra = self.extra_trackers.write().await;
        extra.retain(|t| !trackers.contains(t));
    }

    /// Get all trackers (original + extra)
    pub async fn all_trackers(&self) -> Vec<String> {
        let mut trackers = Vec::new();
        
        if let Some(announce) = &self.metainfo.announce {
            trackers.push(announce.clone());
        }
        for tier in &self.metainfo.announce_list {
            trackers.extend(tier.clone());
        }
        
        let extra = self.extra_trackers.read().await;
        trackers.extend(extra.iter().cloned());
        
        trackers
    }
}

impl Clone for TorrentSession {
    fn clone(&self) -> Self {
        Self {
            metainfo: self.metainfo.clone(),
            peer_id: self.peer_id,
            pieces: self.pieces.clone(),
            state: self.state.clone(),
            stats: self.stats.clone(),
            connected_peers: self.connected_peers.clone(),
            peer_stats: self.peer_stats.clone(),
            available_peers: self.available_peers.clone(),
            download_path: self.download_path.clone(),
            sequential: AtomicBool::new(self.sequential.load(Ordering::Relaxed)),
            stop_tx: self.stop_tx.clone(),
            total_downloaded: AtomicU64::new(self.total_downloaded.load(Ordering::Relaxed)),
            total_uploaded: AtomicU64::new(self.total_uploaded.load(Ordering::Relaxed)),
            port: self.port,
            extra_trackers: self.extra_trackers.clone(),
        }
    }
}
