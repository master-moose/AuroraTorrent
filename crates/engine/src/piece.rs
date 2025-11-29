//! Piece management for BitTorrent downloads
//!
//! Handles piece selection, block requests, verification, and writing

use bitvec::prelude::*;
use bytes::Bytes;
use parking_lot::RwLock;
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tracing::{debug, warn};

/// Standard block size (16 KB)
pub const BLOCK_SIZE: u32 = 16 * 1024;

/// A block within a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockInfo {
    pub piece: u32,
    pub offset: u32,
    pub length: u32,
}

/// State of a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceState {
    Missing,
    Partial,
    Complete,
    Verified,
}

/// Manages piece data and verification
pub struct PieceManager {
    /// Expected SHA1 hashes for each piece
    piece_hashes: Vec<[u8; 20]>,
    /// Size of each piece
    piece_length: u64,
    /// Total torrent size
    total_size: u64,
    /// Number of pieces
    num_pieces: usize,
    /// Bitfield of complete pieces
    pub have: BitVec<u8, Msb0>,
    /// Pieces currently being downloaded
    in_progress: RwLock<HashMap<u32, PieceInProgress>>,
    /// Download directory
    download_path: PathBuf,
    /// Torrent name (used as folder)
    name: String,
    /// File metadata
    files: Vec<FileEntry>,
    /// Priority mode (for streaming)
    sequential: bool,
    /// High priority pieces (for streaming)
    priority_pieces: RwLock<HashSet<u32>>,
}

/// File entry with offset info
#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    length: u64,
    offset: u64,
}

/// A piece being downloaded
struct PieceInProgress {
    blocks: HashMap<u32, Bytes>, // offset -> data
    expected_blocks: u32,
    received_blocks: u32,
}

impl PieceManager {
    pub fn new(
        piece_hashes: Vec<[u8; 20]>,
        piece_length: u64,
        total_size: u64,
        files: Vec<(PathBuf, u64, u64)>,
        download_path: PathBuf,
        name: String,
    ) -> Self {
        let num_pieces = piece_hashes.len();
        let have = bitvec![u8, Msb0; 0; num_pieces];

        let files = files
            .into_iter()
            .map(|(path, length, offset)| FileEntry {
                path,
                length,
                offset,
            })
            .collect();

        Self {
            piece_hashes,
            piece_length,
            total_size,
            num_pieces,
            have,
            in_progress: RwLock::new(HashMap::new()),
            download_path,
            name,
            files,
            sequential: false,
            priority_pieces: RwLock::new(HashSet::new()),
        }
    }

    /// Enable sequential downloading mode
    pub fn set_sequential(&mut self, sequential: bool) {
        self.sequential = sequential;
    }

    /// Set high priority pieces (for streaming)
    pub fn set_priority_pieces(&self, pieces: Vec<u32>) {
        let mut priority = self.priority_pieces.write();
        priority.clear();
        priority.extend(pieces);
    }

    /// Get the size of a specific piece
    pub fn piece_size(&self, piece_idx: u32) -> u32 {
        let idx = piece_idx as usize;
        if idx == self.num_pieces - 1 {
            // Last piece may be smaller
            let remainder = self.total_size % self.piece_length;
            if remainder == 0 {
                self.piece_length as u32
            } else {
                remainder as u32
            }
        } else {
            self.piece_length as u32
        }
    }

    /// Get blocks needed for a piece
    pub fn blocks_for_piece(&self, piece_idx: u32) -> Vec<BlockInfo> {
        let piece_size = self.piece_size(piece_idx);
        let mut blocks = Vec::new();
        let mut offset = 0u32;

        while offset < piece_size {
            let length = std::cmp::min(BLOCK_SIZE, piece_size - offset);
            blocks.push(BlockInfo {
                piece: piece_idx,
                offset,
                length,
            });
            offset += length;
        }

        blocks
    }

    /// Select next piece to download based on strategy
    pub fn select_piece(&self, peer_has: &[u8]) -> Option<u32> {
        let priority = self.priority_pieces.read();

        // First check priority pieces
        for &piece in priority.iter() {
            if !self.have[piece as usize] && Self::peer_has_piece(peer_has, piece) {
                let in_progress = self.in_progress.read();
                if !in_progress.contains_key(&piece) {
                    return Some(piece);
                }
            }
        }
        drop(priority);

        if self.sequential {
            // Sequential: pick lowest missing piece
            for piece in 0..self.num_pieces as u32 {
                if !self.have[piece as usize] && Self::peer_has_piece(peer_has, piece) {
                    let in_progress = self.in_progress.read();
                    if !in_progress.contains_key(&piece) {
                        return Some(piece);
                    }
                }
            }
        } else {
            // Rarest first strategy
            // For simplicity, we'll do random selection from available pieces
            let available: Vec<u32> = (0..self.num_pieces as u32)
                .filter(|&piece| {
                    !self.have[piece as usize]
                        && Self::peer_has_piece(peer_has, piece)
                        && !self.in_progress.read().contains_key(&piece)
                })
                .collect();

            if !available.is_empty() {
                let idx = rand::random::<usize>() % available.len();
                return Some(available[idx]);
            }
        }

        None
    }

    fn peer_has_piece(bitfield: &[u8], piece: u32) -> bool {
        let byte_idx = (piece / 8) as usize;
        let bit_idx = 7 - (piece % 8);
        if byte_idx < bitfield.len() {
            (bitfield[byte_idx] >> bit_idx) & 1 == 1
        } else {
            false
        }
    }

    /// Start downloading a piece
    pub fn start_piece(&self, piece_idx: u32) -> Vec<BlockInfo> {
        let blocks = self.blocks_for_piece(piece_idx);
        let expected_blocks = blocks.len() as u32;

        let mut in_progress = self.in_progress.write();
        in_progress.insert(
            piece_idx,
            PieceInProgress {
                blocks: HashMap::new(),
                expected_blocks,
                received_blocks: 0,
            },
        );

        blocks
    }

    /// Receive a block
    pub fn receive_block(&self, piece_idx: u32, offset: u32, data: Bytes) -> bool {
        let mut in_progress = self.in_progress.write();

        if let Some(piece) = in_progress.get_mut(&piece_idx) {
            if let std::collections::hash_map::Entry::Vacant(e) = piece.blocks.entry(offset) {
                e.insert(data);
                piece.received_blocks += 1;
                return piece.received_blocks >= piece.expected_blocks;
            }
        }

        false
    }

    /// Check if a piece is complete and verify it
    pub async fn verify_and_write_piece(&self, piece_idx: u32) -> Result<bool, std::io::Error> {
        // Get piece data
        let piece_data = {
            let in_progress = self.in_progress.read();
            let piece = match in_progress.get(&piece_idx) {
                Some(p) => p,
                None => return Ok(false),
            };

            // Assemble piece from blocks
            let piece_size = self.piece_size(piece_idx) as usize;
            let mut data = vec![0u8; piece_size];

            for (&offset, block) in &piece.blocks {
                let start = offset as usize;
                let end = std::cmp::min(start + block.len(), piece_size);
                data[start..end].copy_from_slice(&block[..end - start]);
            }

            data
        };

        // Verify SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&piece_data);
        let hash: [u8; 20] = hasher.finalize().into();

        if hash != self.piece_hashes[piece_idx as usize] {
            warn!("Piece {} hash mismatch!", piece_idx);
            // Remove from in_progress so it can be retried
            self.in_progress.write().remove(&piece_idx);
            return Ok(false);
        }

        // Write to disk
        self.write_piece(piece_idx, &piece_data).await?;

        // Remove from in_progress and mark as complete
        self.in_progress.write().remove(&piece_idx);

        debug!("Piece {} verified and written", piece_idx);
        Ok(true)
    }

    /// Write piece data to the correct file(s)
    async fn write_piece(&self, piece_idx: u32, data: &[u8]) -> std::io::Result<()> {
        let piece_start = piece_idx as u64 * self.piece_length;
        let piece_end = piece_start + data.len() as u64;

        for file in &self.files {
            let file_start = file.offset;
            let file_end = file.offset + file.length;

            // Check if this piece overlaps with this file
            if piece_start < file_end && piece_end > file_start {
                let overlap_start = std::cmp::max(piece_start, file_start);
                let overlap_end = std::cmp::min(piece_end, file_end);

                let data_offset = (overlap_start - piece_start) as usize;
                let data_len = (overlap_end - overlap_start) as usize;
                let file_offset = overlap_start - file_start;

                // Ensure parent directories exist
                let full_path = self.download_path.join(&self.name).join(&file.path);
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                // Open file and write (truncate: false to allow partial writes to existing files)
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&full_path)
                    .await?;

                // Pre-allocate file if needed
                let metadata = f.metadata().await?;
                if metadata.len() < file.length {
                    f.set_len(file.length).await?;
                }

                f.seek(SeekFrom::Start(file_offset)).await?;
                f.write_all(&data[data_offset..data_offset + data_len])
                    .await?;
                f.flush().await?;
            }
        }

        Ok(())
    }

    /// Read data from downloaded files (for streaming/seeding)
    pub async fn read_data(&self, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
        let mut result = vec![0u8; length];
        let mut bytes_read = 0usize;
        let end = offset + length as u64;

        for file in &self.files {
            let file_start = file.offset;
            let file_end = file.offset + file.length;

            if offset < file_end && end > file_start {
                let overlap_start = std::cmp::max(offset, file_start);
                let overlap_end = std::cmp::min(end, file_end);

                let result_offset = (overlap_start - offset) as usize;
                let read_len = (overlap_end - overlap_start) as usize;
                let file_offset = overlap_start - file_start;

                let full_path = self.download_path.join(&self.name).join(&file.path);

                if full_path.exists() {
                    let mut f = File::open(&full_path).await?;
                    f.seek(SeekFrom::Start(file_offset)).await?;
                    f.read_exact(&mut result[result_offset..result_offset + read_len])
                        .await?;
                    bytes_read += read_len;
                }
            }
        }

        if bytes_read == length {
            Ok(result)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not all data available",
            ))
        }
    }

    /// Check if a byte range is available (all pieces verified)
    pub fn is_range_available(&self, offset: u64, length: u64) -> bool {
        let start_piece = (offset / self.piece_length) as usize;
        let end_piece = ((offset + length - 1) / self.piece_length) as usize;

        for piece in start_piece..=end_piece {
            if piece < self.num_pieces && !self.have[piece] {
                return false;
            }
        }
        true
    }

    /// Get pieces needed for a byte range
    pub fn pieces_for_range(&self, offset: u64, length: u64) -> Vec<u32> {
        let start_piece = (offset / self.piece_length) as u32;
        let end_piece = ((offset + length - 1) / self.piece_length) as u32;

        (start_piece..=end_piece)
            .filter(|&p| (p as usize) < self.num_pieces && !self.have[p as usize])
            .collect()
    }

    /// Mark a piece as verified (for resuming)
    pub fn mark_verified(&mut self, piece_idx: usize) {
        if piece_idx < self.num_pieces {
            self.have.set(piece_idx, true);
        }
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        let verified = self.have.count_ones();
        verified as f64 / self.num_pieces as f64
    }

    /// Get number of verified pieces
    pub fn verified_count(&self) -> usize {
        self.have.count_ones()
    }

    /// Get total pieces
    pub fn total_pieces(&self) -> usize {
        self.num_pieces
    }

    /// Export bitfield for peer protocol
    pub fn bitfield(&self) -> Vec<u8> {
        self.have.clone().into_vec()
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        self.have.count_ones() == self.num_pieces
    }

    /// Cancel a piece download
    pub fn cancel_piece(&self, piece_idx: u32) {
        self.in_progress.write().remove(&piece_idx);
    }

    /// Verify all pieces from disk (force recheck)
    pub async fn verify_all(&mut self) {
        debug!("Starting full piece verification...");
        
        // Reset all pieces to unverified
        self.have.fill(false);
        self.in_progress.write().clear();
        
        for piece_idx in 0..self.num_pieces {
            let piece_size = self.piece_size(piece_idx as u32) as usize;
            let piece_start = piece_idx as u64 * self.piece_length;
            
            // Try to read piece data from disk
            match self.read_data(piece_start, piece_size).await {
                Ok(data) => {
                    // Verify hash
                    let mut hasher = Sha1::new();
                    hasher.update(&data);
                    let hash: [u8; 20] = hasher.finalize().into();
                    
                    if hash == self.piece_hashes[piece_idx] {
                        self.have.set(piece_idx, true);
                        debug!("Piece {} verified", piece_idx);
                    } else {
                        debug!("Piece {} hash mismatch", piece_idx);
                    }
                }
                Err(_) => {
                    // Piece not available on disk
                    debug!("Piece {} not available on disk", piece_idx);
                }
            }
        }
        
        let verified = self.have.count_ones();
        debug!("Verification complete: {}/{} pieces valid", verified, self.num_pieces);
    }
}
