import { useState } from 'react';
import { motion } from 'framer-motion';
import { X, File, Users, Server, Play, Pause, Trash2, Download } from 'lucide-react';
import { Torrent, FileInfo, PeerInfo, TrackerInfo } from '../types';

interface TorrentDetailsProps {
    torrent: Torrent;
    onClose: () => void;
    onPause: (id: string) => void;
    onResume: (id: string) => void;
    onRemove: (id: string) => void;
    onStream: (id: string) => void;
}

export default function TorrentDetails({ torrent, onClose, onPause, onResume, onRemove, onStream }: TorrentDetailsProps) {
    const [activeTab, setActiveTab] = useState<'files' | 'peers' | 'trackers'>('files');
    const [showConfirmRemove, setShowConfirmRemove] = useState(false);

    const formatSize = (bytes: number) => {
        if (!bytes || bytes === 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    const formatSpeed = (bytes: number) => {
        if (!bytes || bytes === 0) return '0 B/s';
        const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    const tabs = [
        { id: 'files', icon: File, label: 'Files', count: torrent.files?.length || 0 },
        { id: 'peers', icon: Users, label: 'Peers', count: torrent.peers?.length || 0 },
        { id: 'trackers', icon: Server, label: 'Trackers', count: torrent.trackers?.length || 0 },
    ] as const;

    return (
        <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-aurora-void/80 backdrop-blur-sm z-[100] flex items-center justify-center p-4"
            onClick={onClose}
        >
            <motion.div
                initial={{ opacity: 0, scale: 0.95, y: 20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 20 }}
                className="card w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden"
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="p-6 border-b border-aurora-border/50">
                    <div className="flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                            <h2 className="text-xl font-bold text-aurora-text truncate mb-2">
                                {torrent.name}
                            </h2>
                            <div className="flex items-center gap-4 text-sm text-aurora-dim">
                                <span>{formatSize(torrent.total_size)}</span>
                                <span>•</span>
                                <span className={
                                    torrent.status === 'Downloading' || torrent.status === 'Streaming'
                                        ? 'text-aurora-cyan'
                                        : torrent.status === 'Seeding'
                                        ? 'text-aurora-teal'
                                        : ''
                                }>
                                    {torrent.status}
                                </span>
                                <span>•</span>
                                <span>{Math.round(torrent.progress * 100)}%</span>
                            </div>
                        </div>
                        <button
                            onClick={onClose}
                            className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded-lg transition-colors"
                        >
                            <X size={20} />
                        </button>
                    </div>

                    {/* Progress bar */}
                    <div className="mt-4 progress-bar">
                        <div 
                            className="progress-bar-fill"
                            style={{ width: `${torrent.progress * 100}%` }}
                        />
                    </div>

                    {/* Action buttons */}
                    <div className="flex items-center gap-3 mt-4">
                        <button
                            onClick={() => onStream(torrent.id)}
                            className="btn-primary flex items-center gap-2"
                        >
                            <Play size={18} />
                            Stream
                        </button>
                        {torrent.status === 'Downloading' || torrent.status === 'Streaming' ? (
                            <button
                                onClick={() => onPause(torrent.id)}
                                className="btn-secondary flex items-center gap-2"
                            >
                                <Pause size={18} />
                                Pause
                            </button>
                        ) : torrent.status === 'Paused' ? (
                            <button
                                onClick={() => onResume(torrent.id)}
                                className="btn-secondary flex items-center gap-2"
                            >
                                <Download size={18} />
                                Resume
                            </button>
                        ) : null}
                        <button
                            onClick={() => setShowConfirmRemove(true)}
                            className="btn-ghost text-aurora-rose hover:bg-aurora-rose/10 flex items-center gap-2"
                        >
                            <Trash2 size={18} />
                            Remove
                        </button>
                    </div>
                </div>

                {/* Tabs */}
                <div className="flex gap-1 px-6 pt-4 border-b border-aurora-border/50">
                    {tabs.map(({ id, icon: Icon, label, count }) => (
                        <button
                            key={id}
                            onClick={() => setActiveTab(id)}
                            className={`flex items-center gap-2 px-4 py-3 text-sm font-medium rounded-t-lg transition-colors ${
                                activeTab === id
                                    ? 'text-aurora-cyan bg-aurora-night/50 border-b-2 border-aurora-cyan'
                                    : 'text-aurora-dim hover:text-aurora-text'
                            }`}
                        >
                            <Icon size={16} />
                            {label}
                            <span className="text-xs bg-aurora-night px-2 py-0.5 rounded-full">
                                {count}
                            </span>
                        </button>
                    ))}
                </div>

                {/* Tab content */}
                <div className="flex-1 overflow-y-auto p-6">
                    {activeTab === 'files' && (
                        <div className="space-y-2">
                            {torrent.files?.length ? (
                                torrent.files.map((file: FileInfo, i: number) => (
                                    <div 
                                        key={i}
                                        className="flex items-center gap-4 p-3 rounded-lg bg-aurora-night/30 hover:bg-aurora-night/50 transition-colors"
                                    >
                                        <File className="w-5 h-5 text-aurora-dim flex-shrink-0" />
                                        <div className="flex-1 min-w-0">
                                            <p className="text-sm text-aurora-text truncate">
                                                {file.name}
                                            </p>
                                            <div className="flex items-center gap-2 mt-1">
                                                <div className="flex-1 h-1 bg-aurora-night rounded-full overflow-hidden max-w-[200px]">
                                                    <div 
                                                        className="h-full bg-aurora-cyan rounded-full"
                                                        style={{ width: `${(file.progress || 0) * 100}%` }}
                                                    />
                                                </div>
                                                <span className="text-xs text-aurora-dim">
                                                    {Math.round((file.progress || 0) * 100)}%
                                                </span>
                                            </div>
                                        </div>
                                        <span className="text-sm text-aurora-dim">
                                            {formatSize(file.size)}
                                        </span>
                                    </div>
                                ))
                            ) : (
                                <p className="text-aurora-dim text-center py-8">No files</p>
                            )}
                        </div>
                    )}

                    {activeTab === 'peers' && (
                        <div className="space-y-2">
                            {torrent.peers?.length ? (
                                torrent.peers.map((peer: PeerInfo, i: number) => (
                                    <div 
                                        key={i}
                                        className="flex items-center gap-4 p-3 rounded-lg bg-aurora-night/30"
                                    >
                                        <div className="w-2 h-2 rounded-full bg-aurora-teal" />
                                        <div className="flex-1">
                                            <p className="text-sm text-aurora-text font-mono">
                                                {peer.ip}
                                            </p>
                                            <p className="text-xs text-aurora-dim">{peer.client}</p>
                                        </div>
                                        <div className="text-right text-sm">
                                            <p className="text-aurora-cyan">↓ {formatSpeed(peer.down_speed)}</p>
                                            <p className="text-aurora-teal">↑ {formatSpeed(peer.up_speed)}</p>
                                        </div>
                                    </div>
                                ))
                            ) : (
                                <p className="text-aurora-dim text-center py-8">No peers connected</p>
                            )}
                        </div>
                    )}

                    {activeTab === 'trackers' && (
                        <div className="space-y-2">
                            {torrent.trackers?.length ? (
                                torrent.trackers.map((tracker: TrackerInfo, i: number) => (
                                    <div 
                                        key={i}
                                        className="flex items-center gap-4 p-3 rounded-lg bg-aurora-night/30"
                                    >
                                        <div className={`w-2 h-2 rounded-full ${
                                            tracker.status === 'Working' || tracker.status === 'Active'
                                                ? 'bg-aurora-teal'
                                                : 'bg-aurora-muted'
                                        }`} />
                                        <div className="flex-1 min-w-0">
                                            <p className="text-sm text-aurora-text font-mono truncate">
                                                {tracker.url}
                                            </p>
                                        </div>
                                        <span className={`text-xs px-2 py-1 rounded ${
                                            tracker.status === 'Working' || tracker.status === 'Active'
                                                ? 'bg-aurora-teal/10 text-aurora-teal'
                                                : 'bg-aurora-night text-aurora-dim'
                                        }`}>
                                            {tracker.status}
                                        </span>
                                    </div>
                                ))
                            ) : (
                                <p className="text-aurora-dim text-center py-8">No trackers</p>
                            )}
                        </div>
                    )}
                </div>

                {/* Confirm remove dialog */}
                {showConfirmRemove && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        className="absolute inset-0 bg-aurora-void/90 backdrop-blur-sm flex items-center justify-center p-6"
                    >
                        <div className="card p-6 max-w-sm w-full text-center">
                            <Trash2 className="w-12 h-12 text-aurora-rose mx-auto mb-4" />
                            <h3 className="text-lg font-bold mb-2">Remove Torrent?</h3>
                            <p className="text-aurora-dim text-sm mb-6">
                                This will remove the torrent from the list. Downloaded files will not be deleted.
                            </p>
                            <div className="flex gap-3 justify-center">
                                <button
                                    onClick={() => setShowConfirmRemove(false)}
                                    className="btn-secondary"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={() => {
                                        onRemove(torrent.id);
                                        setShowConfirmRemove(false);
                                    }}
                                    className="btn-primary bg-aurora-rose hover:bg-aurora-rose/80"
                                >
                                    Remove
                                </button>
                            </div>
                        </div>
                    </motion.div>
                )}
            </motion.div>
        </motion.div>
    );
}
