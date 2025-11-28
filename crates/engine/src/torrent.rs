//! Torrent file parsing and metadata
//!
//! Handles both .torrent files and magnet links

use crate::bencode::{BencodeError, BencodeValue};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TorrentError {
    #[error("Bencode parse error: {0}")]
    Bencode(#[from] BencodeError),
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Invalid torrent structure")]
    InvalidStructure,
    #[error("Invalid magnet link")]
    InvalidMagnet,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A file within a torrent
#[derive(Debug, Clone)]
pub struct TorrentFile {
    pub path: PathBuf,
    pub length: u64,
    pub offset: u64, // Offset in the concatenated file stream
}

/// Parsed torrent metadata
#[derive(Debug, Clone)]
pub struct TorrentMetainfo {
    /// SHA1 hash of the info dictionary (20 bytes)
    pub info_hash: [u8; 20],
    /// Human-readable name
    pub name: String,
    /// Size of each piece in bytes
    pub piece_length: u64,
    /// SHA1 hashes of all pieces (20 bytes each)
    pub pieces: Vec<[u8; 20]>,
    /// Files in the torrent
    pub files: Vec<TorrentFile>,
    /// Total size of all files
    pub total_size: u64,
    /// Number of pieces
    pub num_pieces: usize,
    /// Announce URL (primary tracker)
    pub announce: Option<String>,
    /// Announce list (multiple trackers)
    pub announce_list: Vec<Vec<String>>,
    /// Is this a private torrent?
    pub private: bool,
    /// Creation date (Unix timestamp)
    pub creation_date: Option<i64>,
    /// Comment
    pub comment: Option<String>,
    /// Created by
    pub created_by: Option<String>,
}

impl TorrentMetainfo {
    /// Parse a .torrent file from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, TorrentError> {
        let (value, _) = BencodeValue::parse(data)?;
        Self::from_bencode(&value, data)
    }

    /// Parse from a bencode value
    fn from_bencode(value: &BencodeValue, raw_data: &[u8]) -> Result<Self, TorrentError> {
        let dict = value.as_dict().ok_or(TorrentError::InvalidStructure)?;

        // Get info dictionary
        let info = value.get("info").ok_or(TorrentError::MissingField("info"))?;
        let info_dict = info.as_dict().ok_or(TorrentError::InvalidStructure)?;

        // Calculate info_hash from raw bytes
        // We need to find the info dict in the raw data and hash it
        let info_hash = Self::calculate_info_hash(raw_data)?;

        // Parse name
        let name = info
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or(TorrentError::MissingField("name"))?
            .to_string();

        // Parse piece length
        let piece_length = info
            .get("piece length")
            .and_then(|v| v.as_integer())
            .ok_or(TorrentError::MissingField("piece length"))? as u64;

        // Parse pieces (concatenated SHA1 hashes)
        let pieces_raw = info
            .get("pieces")
            .and_then(|v| v.as_string())
            .ok_or(TorrentError::MissingField("pieces"))?;

        if pieces_raw.len() % 20 != 0 {
            return Err(TorrentError::InvalidStructure);
        }

        let pieces: Vec<[u8; 20]> = pieces_raw
            .chunks(20)
            .map(|chunk| {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(chunk);
                arr
            })
            .collect();

        // Parse files (single file or multiple files)
        let (files, total_size) = if let Some(length) = info.get("length") {
            // Single file mode
            let length = length.as_integer().ok_or(TorrentError::InvalidStructure)? as u64;
            let file = TorrentFile {
                path: PathBuf::from(&name),
                length,
                offset: 0,
            };
            (vec![file], length)
        } else if let Some(files_list) = info.get("files") {
            // Multiple files mode
            let files_list = files_list.as_list().ok_or(TorrentError::InvalidStructure)?;
            let mut files = Vec::new();
            let mut offset = 0u64;

            for file_entry in files_list {
                let file_dict = file_entry.as_dict().ok_or(TorrentError::InvalidStructure)?;

                let length = file_entry
                    .get("length")
                    .and_then(|v| v.as_integer())
                    .ok_or(TorrentError::MissingField("file length"))? as u64;

                let path_list = file_entry
                    .get("path")
                    .and_then(|v| v.as_list())
                    .ok_or(TorrentError::MissingField("file path"))?;

                let path: PathBuf = std::iter::once(name.clone())
                    .chain(path_list.iter().filter_map(|p| p.as_str().map(String::from)))
                    .collect();

                files.push(TorrentFile {
                    path,
                    length,
                    offset,
                });
                offset += length;
            }

            let total = files.iter().map(|f| f.length).sum();
            (files, total)
        } else {
            return Err(TorrentError::MissingField("length or files"));
        };

        let num_pieces = pieces.len();

        // Parse optional fields
        let announce = value.get("announce").and_then(|v| v.as_str()).map(String::from);

        let announce_list = value
            .get("announce-list")
            .and_then(|v| v.as_list())
            .map(|tiers| {
                tiers
                    .iter()
                    .filter_map(|tier| {
                        tier.as_list().map(|urls| {
                            urls.iter()
                                .filter_map(|u| u.as_str().map(String::from))
                                .collect()
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let private = info
            .get("private")
            .and_then(|v| v.as_integer())
            .map(|v| v == 1)
            .unwrap_or(false);

        let creation_date = value.get("creation date").and_then(|v| v.as_integer());
        let comment = value.get("comment").and_then(|v| v.as_str()).map(String::from);
        let created_by = value.get("created by").and_then(|v| v.as_str()).map(String::from);

        Ok(TorrentMetainfo {
            info_hash,
            name,
            piece_length,
            pieces,
            files,
            total_size,
            num_pieces,
            announce,
            announce_list,
            private,
            creation_date,
            comment,
            created_by,
        })
    }

    /// Calculate info hash from raw torrent data
    fn calculate_info_hash(data: &[u8]) -> Result<[u8; 20], TorrentError> {
        // Find "4:info" in the data and hash from the 'd' after it
        let info_key = b"4:infod";
        let pos = data
            .windows(info_key.len())
            .position(|w| w == info_key)
            .ok_or(TorrentError::MissingField("info"))?;

        let info_start = pos + 6; // Start at 'd'
        
        // Parse to find the matching 'e'
        let (_, consumed) = BencodeValue::parse(&data[info_start..])?;
        let info_bytes = &data[info_start..info_start + consumed];

        let mut hasher = Sha1::new();
        hasher.update(info_bytes);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Create metadata from a magnet link (partial, requires DHT/trackers to complete)
    pub fn from_magnet(magnet: &str) -> Result<MagnetInfo, TorrentError> {
        if !magnet.starts_with("magnet:?") {
            return Err(TorrentError::InvalidMagnet);
        }

        let params: std::collections::HashMap<String, String> = magnet[8..]
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect();

        // Parse info hash from xt parameter
        let xt = params.get("xt").ok_or(TorrentError::InvalidMagnet)?;
        let info_hash = if let Some(hash_hex) = xt.strip_prefix("urn:btih:") {
            if hash_hex.len() == 40 {
                // Hex encoded
                let mut hash = [0u8; 20];
                hex::decode_to_slice(hash_hex, &mut hash)
                    .map_err(|_| TorrentError::InvalidMagnet)?;
                hash
            } else if hash_hex.len() == 32 {
                // Base32 encoded
                Self::decode_base32(hash_hex)?
            } else {
                return Err(TorrentError::InvalidMagnet);
            }
        } else {
            return Err(TorrentError::InvalidMagnet);
        };

        let name = params
            .get("dn")
            .map(|s| urlencoding::decode(s).unwrap_or_default().into_owned())
            .unwrap_or_else(|| hex::encode(info_hash));

        let trackers: Vec<String> = params
            .iter()
            .filter(|(k, _)| k.as_str() == "tr")
            .filter_map(|(_, v)| urlencoding::decode(v).ok().map(|s| s.into_owned()))
            .collect();

        Ok(MagnetInfo {
            info_hash,
            name,
            trackers,
        })
    }

    fn decode_base32(input: &str) -> Result<[u8; 20], TorrentError> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        
        let input = input.to_uppercase();
        let mut result = [0u8; 20];
        let mut buffer = 0u64;
        let mut bits = 0;
        let mut idx = 0;

        for c in input.bytes() {
            let value = ALPHABET
                .iter()
                .position(|&x| x == c)
                .ok_or(TorrentError::InvalidMagnet)? as u64;
            
            buffer = (buffer << 5) | value;
            bits += 5;

            while bits >= 8 && idx < 20 {
                bits -= 8;
                result[idx] = ((buffer >> bits) & 0xFF) as u8;
                idx += 1;
            }
        }

        if idx != 20 {
            return Err(TorrentError::InvalidMagnet);
        }

        Ok(result)
    }

    /// Get info hash as hex string
    pub fn info_hash_hex(&self) -> String {
        hex::encode(self.info_hash)
    }

    /// Get piece index and offset for a byte position
    pub fn piece_for_byte(&self, byte_offset: u64) -> (usize, usize) {
        let piece_idx = (byte_offset / self.piece_length) as usize;
        let piece_offset = (byte_offset % self.piece_length) as usize;
        (piece_idx, piece_offset)
    }

    /// Get the file that contains a given byte offset
    pub fn file_for_byte(&self, byte_offset: u64) -> Option<(usize, &TorrentFile, u64)> {
        for (idx, file) in self.files.iter().enumerate() {
            if byte_offset >= file.offset && byte_offset < file.offset + file.length {
                let offset_in_file = byte_offset - file.offset;
                return Some((idx, file, offset_in_file));
            }
        }
        None
    }

    /// Get the expected size of a piece
    pub fn piece_size(&self, piece_idx: usize) -> usize {
        if piece_idx == self.num_pieces - 1 {
            // Last piece may be smaller
            let remainder = self.total_size % self.piece_length;
            if remainder == 0 {
                self.piece_length as usize
            } else {
                remainder as usize
            }
        } else {
            self.piece_length as usize
        }
    }
}

/// Partial metadata from magnet link
#[derive(Debug, Clone)]
pub struct MagnetInfo {
    pub info_hash: [u8; 20],
    pub name: String,
    pub trackers: Vec<String>,
}

impl MagnetInfo {
    pub fn info_hash_hex(&self) -> String {
        hex::encode(self.info_hash)
    }
}

