//! BitTorrent Tracker Protocol
//!
//! Supports HTTP and UDP trackers

use crate::bencode::BencodeValue;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::debug;
use url::Url;

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid tracker response")]
    InvalidResponse,
    #[error("Tracker failure: {0}")]
    Failure(String),
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("Timeout")]
    Timeout,
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Tracker announce event
#[derive(Debug, Clone, Copy)]
pub enum TrackerEvent {
    None,
    Started,
    Completed,
    Stopped,
}

impl TrackerEvent {
    fn as_str(&self) -> &'static str {
        match self {
            TrackerEvent::None => "",
            TrackerEvent::Started => "started",
            TrackerEvent::Completed => "completed",
            TrackerEvent::Stopped => "stopped",
        }
    }

    fn as_u32(&self) -> u32 {
        match self {
            TrackerEvent::None => 0,
            TrackerEvent::Completed => 1,
            TrackerEvent::Started => 2,
            TrackerEvent::Stopped => 3,
        }
    }
}

/// Parameters for tracker announce
#[derive(Debug, Clone)]
pub struct AnnounceParams {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: TrackerEvent,
    pub compact: bool,
    pub numwant: Option<u32>,
}

/// Tracker announce response
#[derive(Debug, Clone)]
pub struct AnnounceResponse {
    pub interval: u32,
    pub min_interval: Option<u32>,
    pub complete: Option<u32>,   // Seeders
    pub incomplete: Option<u32>, // Leechers
    pub peers: Vec<SocketAddr>,
}

/// Announce to a tracker
pub async fn announce(
    url: &str,
    params: &AnnounceParams,
) -> Result<AnnounceResponse, TrackerError> {
    let parsed = Url::parse(url).map_err(|e| TrackerError::Parse(e.to_string()))?;

    match parsed.scheme() {
        "http" | "https" => announce_http(url, params).await,
        "udp" => announce_udp(&parsed, params).await,
        other => Err(TrackerError::UnsupportedProtocol(other.to_string())),
    }
}

/// HTTP tracker announce
async fn announce_http(
    base_url: &str,
    params: &AnnounceParams,
) -> Result<AnnounceResponse, TrackerError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    // Build query string manually for binary data
    let info_hash_encoded = urlencoding::encode_binary(&params.info_hash);
    let peer_id_encoded = urlencoding::encode_binary(&params.peer_id);

    let mut url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
        base_url,
        info_hash_encoded,
        peer_id_encoded,
        params.port,
        params.uploaded,
        params.downloaded,
        params.left,
        if params.compact { 1 } else { 0 }
    );

    if !params.event.as_str().is_empty() {
        url.push_str(&format!("&event={}", params.event.as_str()));
    }

    if let Some(numwant) = params.numwant {
        url.push_str(&format!("&numwant={}", numwant));
    }

    debug!("HTTP announce: {}", url);

    let response = client.get(&url).send().await?;
    let body = response.bytes().await?;

    let (value, _) = BencodeValue::parse(&body).map_err(|_| TrackerError::InvalidResponse)?;

    // Check for failure
    if let Some(failure) = value.get("failure reason") {
        let msg = failure.as_str().unwrap_or("Unknown error");
        return Err(TrackerError::Failure(msg.to_string()));
    }

    let interval = value
        .get("interval")
        .and_then(|v| v.as_integer())
        .ok_or(TrackerError::InvalidResponse)? as u32;

    let min_interval = value
        .get("min interval")
        .and_then(|v| v.as_integer())
        .map(|v| v as u32);

    let complete = value
        .get("complete")
        .and_then(|v| v.as_integer())
        .map(|v| v as u32);
    let incomplete = value
        .get("incomplete")
        .and_then(|v| v.as_integer())
        .map(|v| v as u32);

    // Parse peers (compact or dictionary format)
    let peers = if let Some(peers_data) = value.get("peers") {
        if let Some(compact_peers) = peers_data.as_string() {
            // Compact format: 6 bytes per peer (4 IP + 2 port)
            parse_compact_peers(compact_peers)
        } else if let Some(peer_list) = peers_data.as_list() {
            // Dictionary format
            peer_list
                .iter()
                .filter_map(|peer| {
                    let ip = peer.get("ip")?.as_str()?;
                    let port = peer.get("port")?.as_integer()? as u16;
                    let addr: Ipv4Addr = ip.parse().ok()?;
                    Some(SocketAddr::V4(SocketAddrV4::new(addr, port)))
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(AnnounceResponse {
        interval,
        min_interval,
        complete,
        incomplete,
        peers,
    })
}

/// UDP tracker announce (BEP 15)
async fn announce_udp(
    url: &Url,
    params: &AnnounceParams,
) -> Result<AnnounceResponse, TrackerError> {
    let host = url.host_str().ok_or(TrackerError::InvalidResponse)?;
    let port = url.port().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    debug!("UDP announce: {}", addr);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(&addr).await?;

    // Step 1: Connect
    let connection_id = udp_connect(&socket).await?;

    // Step 2: Announce
    let response = udp_announce(&socket, connection_id, params).await?;

    Ok(response)
}

async fn udp_connect(socket: &UdpSocket) -> Result<u64, TrackerError> {
    let transaction_id: u32 = rand::random();

    // Connect request: 16 bytes
    let mut request = Vec::with_capacity(16);
    request.extend_from_slice(&0x41727101980u64.to_be_bytes()); // Protocol ID
    request.extend_from_slice(&0u32.to_be_bytes()); // Action: connect
    request.extend_from_slice(&transaction_id.to_be_bytes());

    for attempt in 0..3 {
        socket.send(&request).await?;

        let mut buf = [0u8; 16];
        match timeout(
            Duration::from_secs(5 * (1 << attempt)),
            socket.recv(&mut buf),
        )
        .await
        {
            Ok(Ok(16)) => {
                let action = u32::from_be_bytes(buf[0..4].try_into().unwrap());
                let recv_transaction_id = u32::from_be_bytes(buf[4..8].try_into().unwrap());
                let connection_id = u64::from_be_bytes(buf[8..16].try_into().unwrap());

                if action != 0 || recv_transaction_id != transaction_id {
                    continue;
                }

                return Ok(connection_id);
            }
            _ => continue,
        }
    }

    Err(TrackerError::Timeout)
}

async fn udp_announce(
    socket: &UdpSocket,
    connection_id: u64,
    params: &AnnounceParams,
) -> Result<AnnounceResponse, TrackerError> {
    let transaction_id: u32 = rand::random();

    // Announce request: 98 bytes
    let mut request = Vec::with_capacity(98);
    request.extend_from_slice(&connection_id.to_be_bytes());
    request.extend_from_slice(&1u32.to_be_bytes()); // Action: announce
    request.extend_from_slice(&transaction_id.to_be_bytes());
    request.extend_from_slice(&params.info_hash);
    request.extend_from_slice(&params.peer_id);
    request.extend_from_slice(&params.downloaded.to_be_bytes());
    request.extend_from_slice(&params.left.to_be_bytes());
    request.extend_from_slice(&params.uploaded.to_be_bytes());
    request.extend_from_slice(&params.event.as_u32().to_be_bytes());
    request.extend_from_slice(&0u32.to_be_bytes()); // IP (default)
    request.extend_from_slice(&rand::random::<u32>().to_be_bytes()); // Key
    request.extend_from_slice(&params.numwant.unwrap_or(50).to_be_bytes());
    request.extend_from_slice(&params.port.to_be_bytes());

    for attempt in 0..3 {
        socket.send(&request).await?;

        let mut buf = vec![0u8; 1024];
        match timeout(
            Duration::from_secs(5 * (1 << attempt)),
            socket.recv(&mut buf),
        )
        .await
        {
            Ok(Ok(n)) if n >= 20 => {
                let action = u32::from_be_bytes(buf[0..4].try_into().unwrap());
                let recv_transaction_id = u32::from_be_bytes(buf[4..8].try_into().unwrap());

                if action != 1 || recv_transaction_id != transaction_id {
                    continue;
                }

                let interval = u32::from_be_bytes(buf[8..12].try_into().unwrap());
                let incomplete = u32::from_be_bytes(buf[12..16].try_into().unwrap());
                let complete = u32::from_be_bytes(buf[16..20].try_into().unwrap());

                // Parse peers (6 bytes each)
                let peers = parse_compact_peers(&buf[20..n]);

                return Ok(AnnounceResponse {
                    interval,
                    min_interval: None,
                    complete: Some(complete),
                    incomplete: Some(incomplete),
                    peers,
                });
            }
            _ => continue,
        }
    }

    Err(TrackerError::Timeout)
}

/// Parse compact peer format (6 bytes per peer: 4 IP + 2 port)
fn parse_compact_peers(data: &[u8]) -> Vec<SocketAddr> {
    data.chunks_exact(6)
        .map(|chunk| {
            let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            SocketAddr::V4(SocketAddrV4::new(ip, port))
        })
        .collect()
}
