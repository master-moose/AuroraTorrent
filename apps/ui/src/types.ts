export interface FileInfo {
    name: string;
    size: number;
    progress: number;
}

export interface PeerInfo {
    ip: string;
    client: string;
    down_speed: number;
    up_speed: number;
}

export interface TrackerInfo {
    url: string;
    status: string;
}

export interface Torrent {
    id: string;
    name: string;
    status: string;
    progress: number;
    download_speed: number;
    upload_speed: number;
    total_size: number;
    files: FileInfo[];
    peers: PeerInfo[];
    trackers: TrackerInfo[];
}
