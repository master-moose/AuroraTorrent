//! BitTorrent Peer Wire Protocol implementation
//!
//! Handles peer connections, handshakes, and message exchange

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

/// Protocol identifier
const PROTOCOL: &[u8] = b"BitTorrent protocol";

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageId {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
    // Extended protocol
    Port = 9,
}

impl TryFrom<u8> for MessageId {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageId::Choke),
            1 => Ok(MessageId::Unchoke),
            2 => Ok(MessageId::Interested),
            3 => Ok(MessageId::NotInterested),
            4 => Ok(MessageId::Have),
            5 => Ok(MessageId::Bitfield),
            6 => Ok(MessageId::Request),
            7 => Ok(MessageId::Piece),
            8 => Ok(MessageId::Cancel),
            9 => Ok(MessageId::Port),
            other => Err(other),
        }
    }
}

/// A peer wire protocol message
#[derive(Debug, Clone)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have { piece_index: u32 },
    Bitfield { bitfield: Vec<u8> },
    Request { index: u32, begin: u32, length: u32 },
    Piece { index: u32, begin: u32, block: Bytes },
    Cancel { index: u32, begin: u32, length: u32 },
    Port { port: u16 },
}

impl PeerMessage {
    /// Encode message to bytes
    pub fn encode(&self) -> Bytes {
        match self {
            PeerMessage::KeepAlive => {
                let mut buf = BytesMut::with_capacity(4);
                buf.put_u32(0);
                buf.freeze()
            }
            PeerMessage::Choke => Self::encode_simple(MessageId::Choke),
            PeerMessage::Unchoke => Self::encode_simple(MessageId::Unchoke),
            PeerMessage::Interested => Self::encode_simple(MessageId::Interested),
            PeerMessage::NotInterested => Self::encode_simple(MessageId::NotInterested),
            PeerMessage::Have { piece_index } => {
                let mut buf = BytesMut::with_capacity(9);
                buf.put_u32(5);
                buf.put_u8(MessageId::Have as u8);
                buf.put_u32(*piece_index);
                buf.freeze()
            }
            PeerMessage::Bitfield { bitfield } => {
                let mut buf = BytesMut::with_capacity(5 + bitfield.len());
                buf.put_u32(1 + bitfield.len() as u32);
                buf.put_u8(MessageId::Bitfield as u8);
                buf.put_slice(bitfield);
                buf.freeze()
            }
            PeerMessage::Request { index, begin, length } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u32(13);
                buf.put_u8(MessageId::Request as u8);
                buf.put_u32(*index);
                buf.put_u32(*begin);
                buf.put_u32(*length);
                buf.freeze()
            }
            PeerMessage::Piece { index, begin, block } => {
                let mut buf = BytesMut::with_capacity(13 + block.len());
                buf.put_u32(9 + block.len() as u32);
                buf.put_u8(MessageId::Piece as u8);
                buf.put_u32(*index);
                buf.put_u32(*begin);
                buf.put_slice(block);
                buf.freeze()
            }
            PeerMessage::Cancel { index, begin, length } => {
                let mut buf = BytesMut::with_capacity(17);
                buf.put_u32(13);
                buf.put_u8(MessageId::Cancel as u8);
                buf.put_u32(*index);
                buf.put_u32(*begin);
                buf.put_u32(*length);
                buf.freeze()
            }
            PeerMessage::Port { port } => {
                let mut buf = BytesMut::with_capacity(7);
                buf.put_u32(3);
                buf.put_u8(MessageId::Port as u8);
                buf.put_u16(*port);
                buf.freeze()
            }
        }
    }

    fn encode_simple(id: MessageId) -> Bytes {
        let mut buf = BytesMut::with_capacity(5);
        buf.put_u32(1);
        buf.put_u8(id as u8);
        buf.freeze()
    }

    /// Decode message from bytes (excluding length prefix)
    pub fn decode(mut data: Bytes) -> io::Result<Self> {
        if data.is_empty() {
            return Ok(PeerMessage::KeepAlive);
        }

        let id = data.get_u8();
        match MessageId::try_from(id) {
            Ok(MessageId::Choke) => Ok(PeerMessage::Choke),
            Ok(MessageId::Unchoke) => Ok(PeerMessage::Unchoke),
            Ok(MessageId::Interested) => Ok(PeerMessage::Interested),
            Ok(MessageId::NotInterested) => Ok(PeerMessage::NotInterested),
            Ok(MessageId::Have) => {
                if data.remaining() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Have too short"));
                }
                Ok(PeerMessage::Have {
                    piece_index: data.get_u32(),
                })
            }
            Ok(MessageId::Bitfield) => Ok(PeerMessage::Bitfield {
                bitfield: data.to_vec(),
            }),
            Ok(MessageId::Request) => {
                if data.remaining() < 12 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Request too short"));
                }
                Ok(PeerMessage::Request {
                    index: data.get_u32(),
                    begin: data.get_u32(),
                    length: data.get_u32(),
                })
            }
            Ok(MessageId::Piece) => {
                if data.remaining() < 8 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Piece too short"));
                }
                let index = data.get_u32();
                let begin = data.get_u32();
                Ok(PeerMessage::Piece {
                    index,
                    begin,
                    block: data,
                })
            }
            Ok(MessageId::Cancel) => {
                if data.remaining() < 12 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Cancel too short"));
                }
                Ok(PeerMessage::Cancel {
                    index: data.get_u32(),
                    begin: data.get_u32(),
                    length: data.get_u32(),
                })
            }
            Ok(MessageId::Port) => {
                if data.remaining() < 2 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Port too short"));
                }
                Ok(PeerMessage::Port {
                    port: data.get_u16(),
                })
            }
            Err(id) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown message id: {}", id),
            )),
        }
    }
}

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerState {
    /// Are we choking the peer?
    pub am_choking: bool,
    /// Are we interested in the peer?
    pub am_interested: bool,
    /// Is the peer choking us?
    pub peer_choking: bool,
    /// Is the peer interested in us?
    pub peer_interested: bool,
}

impl Default for PeerState {
    fn default() -> Self {
        Self {
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
        }
    }
}

/// Active peer connection
pub struct PeerConnection {
    pub addr: SocketAddr,
    pub peer_id: [u8; 20],
    pub state: PeerState,
    pub bitfield: Vec<u8>,
    stream: TcpStream,
    pub download_rate: u64,
    pub upload_rate: u64,
    bytes_downloaded: u64,
    bytes_uploaded: u64,
}

impl PeerConnection {
    /// Perform handshake with peer
    pub async fn connect(
        addr: SocketAddr,
        info_hash: [u8; 20],
        our_peer_id: [u8; 20],
        timeout_secs: u64,
    ) -> io::Result<Self> {
        debug!("Connecting to peer: {}", addr);

        let stream = timeout(
            Duration::from_secs(timeout_secs),
            TcpStream::connect(addr),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Connection timeout"))??;

        let mut conn = Self {
            addr,
            peer_id: [0u8; 20],
            state: PeerState::default(),
            bitfield: Vec::new(),
            stream,
            download_rate: 0,
            upload_rate: 0,
            bytes_downloaded: 0,
            bytes_uploaded: 0,
        };

        conn.handshake(info_hash, our_peer_id).await?;
        Ok(conn)
    }

    /// Accept incoming connection
    pub async fn accept(
        mut stream: TcpStream,
        addr: SocketAddr,
        info_hash: [u8; 20],
        our_peer_id: [u8; 20],
    ) -> io::Result<Self> {
        let mut conn = Self {
            addr,
            peer_id: [0u8; 20],
            state: PeerState::default(),
            bitfield: Vec::new(),
            stream,
            download_rate: 0,
            upload_rate: 0,
            bytes_downloaded: 0,
            bytes_uploaded: 0,
        };

        // For incoming, we receive handshake first
        conn.receive_handshake(info_hash).await?;
        conn.send_handshake(info_hash, our_peer_id).await?;
        
        Ok(conn)
    }

    async fn handshake(&mut self, info_hash: [u8; 20], our_peer_id: [u8; 20]) -> io::Result<()> {
        self.send_handshake(info_hash, our_peer_id).await?;
        self.receive_handshake(info_hash).await?;
        Ok(())
    }

    async fn send_handshake(&mut self, info_hash: [u8; 20], our_peer_id: [u8; 20]) -> io::Result<()> {
        let mut handshake = Vec::with_capacity(68);
        handshake.push(PROTOCOL.len() as u8);
        handshake.extend_from_slice(PROTOCOL);
        handshake.extend_from_slice(&[0u8; 8]); // Reserved bytes
        handshake.extend_from_slice(&info_hash);
        handshake.extend_from_slice(&our_peer_id);

        self.stream.write_all(&handshake).await?;
        Ok(())
    }

    async fn receive_handshake(&mut self, expected_info_hash: [u8; 20]) -> io::Result<()> {
        let mut buf = [0u8; 68];
        self.stream.read_exact(&mut buf).await?;

        let pstrlen = buf[0] as usize;
        if pstrlen != PROTOCOL.len() || &buf[1..1 + pstrlen] != PROTOCOL {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid protocol identifier",
            ));
        }

        let info_hash_start = 1 + pstrlen + 8;
        let received_info_hash: [u8; 20] = buf[info_hash_start..info_hash_start + 20]
            .try_into()
            .unwrap();

        if received_info_hash != expected_info_hash {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Info hash mismatch",
            ));
        }

        self.peer_id.copy_from_slice(&buf[info_hash_start + 20..]);
        Ok(())
    }

    /// Send a message to the peer
    pub async fn send(&mut self, msg: PeerMessage) -> io::Result<()> {
        let bytes = msg.encode();
        self.stream.write_all(&bytes).await?;
        
        if let PeerMessage::Piece { block, .. } = &msg {
            self.bytes_uploaded += block.len() as u64;
        }
        
        Ok(())
    }

    /// Receive a message from the peer
    pub async fn recv(&mut self) -> io::Result<PeerMessage> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len == 0 {
            return Ok(PeerMessage::KeepAlive);
        }

        // Sanity check - max piece size + headers
        if len > 16 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Message too large",
            ));
        }

        // Read message body
        let mut body = vec![0u8; len];
        self.stream.read_exact(&mut body).await?;
        
        let msg = PeerMessage::decode(Bytes::from(body))?;
        
        if let PeerMessage::Piece { block, .. } = &msg {
            self.bytes_downloaded += block.len() as u64;
        }
        
        Ok(msg)
    }

    /// Receive with timeout
    pub async fn recv_timeout(&mut self, duration: Duration) -> io::Result<PeerMessage> {
        timeout(duration, self.recv())
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Receive timeout"))?
    }

    /// Update state based on received message
    pub fn handle_message(&mut self, msg: &PeerMessage) {
        match msg {
            PeerMessage::Choke => self.state.peer_choking = true,
            PeerMessage::Unchoke => self.state.peer_choking = false,
            PeerMessage::Interested => self.state.peer_interested = true,
            PeerMessage::NotInterested => self.state.peer_interested = false,
            PeerMessage::Bitfield { bitfield } => {
                self.bitfield = bitfield.clone();
            }
            PeerMessage::Have { piece_index } => {
                let byte_idx = (*piece_index / 8) as usize;
                let bit_idx = 7 - (*piece_index % 8);
                if byte_idx < self.bitfield.len() {
                    self.bitfield[byte_idx] |= 1 << bit_idx;
                } else {
                    // Extend bitfield if needed
                    self.bitfield.resize(byte_idx + 1, 0);
                    self.bitfield[byte_idx] |= 1 << bit_idx;
                }
            }
            _ => {}
        }
    }

    /// Check if peer has a piece
    pub fn has_piece(&self, piece_idx: u32) -> bool {
        let byte_idx = (piece_idx / 8) as usize;
        let bit_idx = 7 - (piece_idx % 8);
        if byte_idx < self.bitfield.len() {
            (self.bitfield[byte_idx] >> bit_idx) & 1 == 1
        } else {
            false
        }
    }

    /// Get client name from peer ID
    pub fn client_name(&self) -> String {
        // Try to parse Azureus-style peer ID (-XX0000-)
        if self.peer_id[0] == b'-' && self.peer_id[7] == b'-' {
            let client_id = String::from_utf8_lossy(&self.peer_id[1..3]);
            return match client_id.as_ref() {
                "AZ" => "Azureus".to_string(),
                "BC" => "BitComet".to_string(),
                "BT" => "BitTorrent".to_string(),
                "DE" => "Deluge".to_string(),
                "LT" => "libtorrent".to_string(),
                "TR" => "Transmission".to_string(),
                "UT" => "µTorrent".to_string(),
                "qB" => "qBittorrent".to_string(),
                "AU" => "Aurora".to_string(),
                _ => format!("Unknown ({})", client_id),
            };
        }
        "Unknown".to_string()
    }

    pub fn downloaded(&self) -> u64 {
        self.bytes_downloaded
    }

    pub fn uploaded(&self) -> u64 {
        self.bytes_uploaded
    }
}

/// Generate our peer ID
pub fn generate_peer_id() -> [u8; 20] {
    let mut id = [0u8; 20];
    // Azureus-style: -AU0100- followed by random bytes
    id[0..8].copy_from_slice(b"-AU0100-");
    for byte in &mut id[8..] {
        *byte = rand::random();
    }
    id
}

