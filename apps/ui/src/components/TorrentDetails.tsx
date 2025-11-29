import { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
    X, File, Users, Server, Play, Pause, Square, Trash2, Download,
    ChevronUp, ChevronDown, ChevronsUp, ChevronsDown, Check,
    Copy, Activity, Shield
} from 'lucide-react';
import { Torrent, FileInfo, PeerInfo, TrackerInfo, FilePriority } from '../types';
import { setFilePriority, queueMoveUp, queueMoveDown, queueMoveTop, queueMoveBottom, stopTorrent } from '../rpc';
import PieceBar from './PieceBar';
import SpeedGraph, { useSpeedSamples } from './SpeedGraph';

interface TorrentDetailsProps {
    torrent: Torrent;
    onClose: () => void;
    onPause: (id: string) => void;
    onResume: (id: string) => void;
    onRemove: (id: string, deleteFiles?: boolean) => void;
    onStream: (id: string) => void;
    onRefresh?: () => void;
}

export default function TorrentDetails({ 
    torrent, 
    onClose, 
    onPause, 
    onResume, 
    onRemove, 
    onStream,
    onRefresh 
}: TorrentDetailsProps) {
    const [activeTab, setActiveTab] = useState<'general' | 'files' | 'peers' | 'trackers' | 'speed'>('general');
    const [showConfirmRemove, setShowConfirmRemove] = useState(false);
    const [deleteFilesOnRemove, setDeleteFilesOnRemove] = useState(false);
    const [selectedFiles, setSelectedFiles] = useState<Set<number>>(new Set());
    const [copied, setCopied] = useState<string | null>(null);

    // Speed samples for this torrent
    const speedSamples = useSpeedSamples(torrent.download_speed, torrent.upload_speed);

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

    const formatEta = (seconds: number) => {
        if (!seconds || seconds === 0) return '∞';
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
        if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
        return `${Math.floor(seconds / 86400)}d ${Math.floor((seconds % 86400) / 3600)}h`;
    };

    const formatDate = (timestamp: number) => {
        if (!timestamp) return 'Unknown';
        return new Date(timestamp * 1000).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    };

    const formatDuration = (seconds: number) => {
        if (!seconds) return '0s';
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        return `${hours}h ${minutes}m`;
    };

    const getPriorityColor = (priority: FilePriority) => {
        switch (priority) {
            case 'High': return 'text-aurora-rose bg-aurora-rose/10';
            case 'Normal': return 'text-aurora-cyan bg-aurora-cyan/10';
            case 'Low': return 'text-aurora-muted bg-aurora-muted/10';
            case 'Skip': return 'text-aurora-dim bg-aurora-night/50 line-through';
            default: return 'text-aurora-dim';
        }
    };

    const handleSetFilePriority = async (fileIndex: number, priority: FilePriority) => {
        await setFilePriority(torrent.id, fileIndex, priority);
        onRefresh?.();
    };

    const handleSelectAll = () => {
        if (selectedFiles.size === torrent.files.length) {
            setSelectedFiles(new Set());
        } else {
            setSelectedFiles(new Set(torrent.files.map((_, i) => i)));
        }
    };

    const handleFileSelect = (index: number) => {
        const newSelected = new Set(selectedFiles);
        if (newSelected.has(index)) {
            newSelected.delete(index);
        } else {
            newSelected.add(index);
        }
        setSelectedFiles(newSelected);
    };

    const handleBulkPriority = async (priority: FilePriority) => {
        for (const index of selectedFiles) {
            await setFilePriority(torrent.id, index, priority);
        }
        setSelectedFiles(new Set());
        onRefresh?.();
    };

    const handleQueueMove = async (action: 'up' | 'down' | 'top' | 'bottom') => {
        switch (action) {
            case 'up': await queueMoveUp(torrent.id); break;
            case 'down': await queueMoveDown(torrent.id); break;
            case 'top': await queueMoveTop(torrent.id); break;
            case 'bottom': await queueMoveBottom(torrent.id); break;
        }
        onRefresh?.();
    };

    const handleStop = async () => {
        await stopTorrent(torrent.id);
        onRefresh?.();
    };

    const handleCopy = async (text: string, label: string) => {
        await navigator.clipboard.writeText(text);
        setCopied(label);
        setTimeout(() => setCopied(null), 2000);
    };

    const magnetUri = torrent.magnet_uri || `magnet:?xt=urn:btih:${torrent.id}&dn=${encodeURIComponent(torrent.name)}`;

    const isActive = torrent.status === 'Downloading' || torrent.status === 'Streaming' || torrent.status === 'Seeding';
    const isPaused = torrent.status === 'Paused';
    const isStopped = torrent.status === 'Stopped';

    const tabs = [
        { id: 'general', icon: Activity, label: 'General' },
        { id: 'files', icon: File, label: 'Files', count: torrent.files?.length || 0 },
        { id: 'peers', icon: Users, label: 'Peers', count: torrent.peers?.length || 0 },
        { id: 'trackers', icon: Server, label: 'Trackers', count: torrent.trackers?.length || 0 },
        { id: 'speed', icon: Activity, label: 'Speed' },
    ] as const;

    // Count peers by type
    const peerStats = useMemo(() => {
        let seeds = 0, leechers = 0;
        for (const peer of torrent.peers || []) {
            if (peer.progress >= 1) seeds++;
            else leechers++;
        }
        return { seeds, leechers, total: torrent.peers?.length || 0 };
    }, [torrent.peers]);

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
                className="card w-full max-w-5xl max-h-[90vh] flex flex-col overflow-hidden"
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="p-6 border-b border-aurora-border/50">
                    <div className="flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                            <h2 className="text-xl font-bold text-aurora-text truncate mb-2">
                                {torrent.name}
                            </h2>
                            <div className="flex items-center gap-4 text-sm text-aurora-dim flex-wrap">
                                <span>{formatSize(torrent.total_size)}</span>
                                <span>•</span>
                                <span className={
                                    isActive
                                        ? 'text-aurora-cyan'
                                        : torrent.status === 'Seeding'
                                        ? 'text-aurora-teal'
                                        : isPaused
                                        ? 'text-aurora-muted'
                                        : ''
                                }>
                                    {torrent.status}
                                </span>
                                <span>•</span>
                                <span>{Math.round(torrent.progress * 100)}%</span>
                                {torrent.eta > 0 && torrent.progress < 1 && (
                                    <>
                                        <span>•</span>
                                        <span>ETA: {formatEta(torrent.eta)}</span>
                                    </>
                                )}
                                {torrent.category && (
                                    <>
                                        <span>•</span>
                                        <span className="px-2 py-0.5 bg-aurora-violet/20 text-aurora-violet rounded text-xs">
                                            {torrent.category}
                                        </span>
                                    </>
                                )}
                                {torrent.is_private && (
                                    <span className="flex items-center gap-1 text-aurora-rose">
                                        <Shield size={12} />
                                        Private
                                    </span>
                                )}
                            </div>
                        </div>
                        <button
                            onClick={onClose}
                            className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded-lg transition-colors"
                        >
                            <X size={20} />
                        </button>
                    </div>

                    {/* Progress bar with piece visualization */}
                    <div className="mt-4">
                        {torrent.piece_states && torrent.piece_states.length > 0 ? (
                            <PieceBar 
                                pieces={torrent.piece_states} 
                                height={24}
                                showTooltip={true}
                            />
                        ) : (
                            <div className="progress-bar">
                                <div 
                                    className="progress-bar-fill"
                                    style={{ width: `${torrent.progress * 100}%` }}
                                />
                            </div>
                        )}
                    </div>

                    {/* Stats row */}
                    <div className="flex items-center gap-6 mt-3 text-sm flex-wrap">
                        <div className="flex items-center gap-2">
                            <span className="text-aurora-cyan">↓ {formatSpeed(torrent.download_speed)}</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <span className="text-aurora-teal">↑ {formatSpeed(torrent.upload_speed)}</span>
                        </div>
                        <div className="text-aurora-dim">
                            Queue: #{torrent.queue_position + 1}
                        </div>
                        <div className="text-aurora-dim">
                            Ratio: {torrent.ratio?.toFixed(2) || '0.00'}
                        </div>
                        <div className="text-aurora-dim">
                            Seeds: {peerStats.seeds} | Leechers: {peerStats.leechers}
                        </div>
                    </div>

                    {/* Action buttons */}
                    <div className="flex items-center gap-3 mt-4 flex-wrap">
                        <button
                            onClick={() => onStream(torrent.id)}
                            className="btn-primary flex items-center gap-2"
                            disabled={torrent.progress === 0}
                        >
                            <Play size={18} />
                            Stream
                        </button>
                        
                        {isActive ? (
                            <button
                                onClick={() => onPause(torrent.id)}
                                className="btn-secondary flex items-center gap-2"
                            >
                                <Pause size={18} />
                                Pause
                            </button>
                        ) : isPaused || isStopped ? (
                            <button
                                onClick={() => onResume(torrent.id)}
                                className="btn-secondary flex items-center gap-2"
                            >
                                <Download size={18} />
                                Resume
                            </button>
                        ) : null}

                        {(isActive || isPaused) && (
                            <button
                                onClick={handleStop}
                                className="btn-ghost flex items-center gap-2"
                            >
                                <Square size={18} />
                                Stop
                            </button>
                        )}

                        <button
                            onClick={() => handleCopy(magnetUri, 'magnet')}
                            className="btn-ghost flex items-center gap-2"
                            title="Copy magnet link"
                        >
                            <Copy size={18} />
                            {copied === 'magnet' ? 'Copied!' : 'Magnet'}
                        </button>

                        <button
                            onClick={() => handleCopy(torrent.id, 'hash')}
                            className="btn-ghost flex items-center gap-2"
                            title="Copy info hash"
                        >
                            <Copy size={18} />
                            {copied === 'hash' ? 'Copied!' : 'Hash'}
                        </button>

                        {/* Queue controls */}
                        <div className="flex items-center gap-1 ml-auto border-l border-aurora-border/50 pl-3">
                            <button
                                onClick={() => handleQueueMove('top')}
                                className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded transition-colors"
                                title="Move to top"
                            >
                                <ChevronsUp size={18} />
                            </button>
                            <button
                                onClick={() => handleQueueMove('up')}
                                className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded transition-colors"
                                title="Move up"
                            >
                                <ChevronUp size={18} />
                            </button>
                            <button
                                onClick={() => handleQueueMove('down')}
                                className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded transition-colors"
                                title="Move down"
                            >
                                <ChevronDown size={18} />
                            </button>
                            <button
                                onClick={() => handleQueueMove('bottom')}
                                className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded transition-colors"
                                title="Move to bottom"
                            >
                                <ChevronsDown size={18} />
                            </button>
                        </div>

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
                <div className="flex gap-1 px-6 pt-4 border-b border-aurora-border/50 overflow-x-auto">
                    {tabs.map((tab) => {
                        const Icon = tab.icon;
                        const count = 'count' in tab ? tab.count : undefined;
                        return (
                            <button
                                key={tab.id}
                                onClick={() => setActiveTab(tab.id)}
                                className={`flex items-center gap-2 px-4 py-3 text-sm font-medium rounded-t-lg transition-colors whitespace-nowrap ${
                                    activeTab === tab.id
                                        ? 'text-aurora-cyan bg-aurora-night/50 border-b-2 border-aurora-cyan'
                                        : 'text-aurora-dim hover:text-aurora-text'
                                }`}
                            >
                                <Icon size={16} />
                                {tab.label}
                                {count !== undefined && (
                                    <span className="text-xs bg-aurora-night px-2 py-0.5 rounded-full">
                                        {count}
                                    </span>
                                )}
                            </button>
                        );
                    })}
                </div>

                {/* Tab content */}
                <div className="flex-1 overflow-y-auto p-6">
                    {/* General Tab */}
                    {activeTab === 'general' && (
                        <div className="grid grid-cols-2 lg:grid-cols-3 gap-4">
                            <InfoCard label="Downloaded" value={formatSize(torrent.downloaded || 0)} />
                            <InfoCard label="Uploaded" value={formatSize(torrent.uploaded || 0)} />
                            <InfoCard label="Share Ratio" value={torrent.ratio?.toFixed(3) || '0.000'} />
                            <InfoCard label="Downloaded (Session)" value={formatSize(torrent.downloaded_session || 0)} />
                            <InfoCard label="Uploaded (Session)" value={formatSize(torrent.uploaded_session || 0)} />
                            <InfoCard label="Wasted" value={formatSize(torrent.wasted || 0)} />
                            <InfoCard label="Seeds" value={`${torrent.connected_seeds || 0} (${torrent.seeds || 0} total)`} />
                            <InfoCard label="Leechers" value={`${torrent.connected_leechers || 0} (${torrent.leechers || 0} total)`} />
                            <InfoCard label="Amount Left" value={formatSize(torrent.amount_left || 0)} />
                            <InfoCard label="Pieces" value={`${torrent.num_pieces || 0} × ${formatSize(torrent.piece_size || 0)}`} />
                            <InfoCard label="Added On" value={formatDate(torrent.added_on)} />
                            <InfoCard label="Completed On" value={torrent.completed_on ? formatDate(torrent.completed_on) : 'N/A'} />
                            <InfoCard label="Seeding Time" value={formatDuration(torrent.seeding_time || 0)} />
                            <InfoCard label="Last Activity" value={torrent.last_activity ? formatDuration(torrent.last_activity) + ' ago' : 'N/A'} />
                            
                            <div className="col-span-full p-4 rounded-lg bg-aurora-night/30">
                                <p className="text-xs text-aurora-dim mb-1">Save Path</p>
                                <p className="text-sm font-mono text-aurora-text break-all">{torrent.save_path || 'Unknown'}</p>
                            </div>
                            
                            <div className="col-span-full p-4 rounded-lg bg-aurora-night/30">
                                <p className="text-xs text-aurora-dim mb-1">Info Hash</p>
                                <p className="text-sm font-mono text-aurora-text break-all">{torrent.id}</p>
                            </div>
                            
                            {torrent.comment && (
                                <div className="col-span-full p-4 rounded-lg bg-aurora-night/30">
                                    <p className="text-xs text-aurora-dim mb-1">Comment</p>
                                    <p className="text-sm text-aurora-text">{torrent.comment}</p>
                                </div>
                            )}
                            
                            {torrent.created_by && (
                                <InfoCard label="Created By" value={torrent.created_by} />
                            )}
                            
                            {torrent.creation_date && (
                                <InfoCard label="Creation Date" value={formatDate(torrent.creation_date)} />
                            )}
                        </div>
                    )}

                    {/* Files Tab */}
                    {activeTab === 'files' && (
                        <div className="space-y-2">
                            {/* Bulk actions */}
                            {selectedFiles.size > 0 && (
                                <div className="flex items-center gap-2 p-3 rounded-lg bg-aurora-cyan/10 border border-aurora-cyan/30 mb-4">
                                    <span className="text-sm text-aurora-cyan">
                                        {selectedFiles.size} file(s) selected
                                    </span>
                                    <div className="flex-1" />
                                    <select
                                        className="input py-1 px-2 text-sm"
                                        onChange={(e) => {
                                            if (e.target.value) {
                                                handleBulkPriority(e.target.value as FilePriority);
                                                e.target.value = '';
                                            }
                                        }}
                                        defaultValue=""
                                    >
                                        <option value="" disabled>Set priority...</option>
                                        <option value="High">High</option>
                                        <option value="Normal">Normal</option>
                                        <option value="Low">Low</option>
                                        <option value="Skip">Skip (Don't download)</option>
                                    </select>
                                </div>
                            )}

                            {/* Select all header */}
                            {torrent.files?.length > 0 && (
                                <div className="flex items-center gap-3 px-3 py-2 text-sm text-aurora-dim border-b border-aurora-border/30">
                                    <button
                                        onClick={handleSelectAll}
                                        className={`w-5 h-5 rounded border flex items-center justify-center transition-colors ${
                                            selectedFiles.size === torrent.files.length
                                                ? 'bg-aurora-cyan border-aurora-cyan text-aurora-void'
                                                : 'border-aurora-border hover:border-aurora-cyan'
                                        }`}
                                    >
                                        {selectedFiles.size === torrent.files.length && <Check size={14} />}
                                    </button>
                                    <span className="flex-1">Name</span>
                                    <span className="w-24 text-right">Size</span>
                                    <span className="w-20 text-right">Progress</span>
                                    <span className="w-24 text-center">Priority</span>
                                </div>
                            )}

                            {torrent.files?.length ? (
                                torrent.files.map((file: FileInfo, i: number) => (
                                    <div 
                                        key={i}
                                        className={`flex items-center gap-3 p-3 rounded-lg transition-colors ${
                                            selectedFiles.has(i) 
                                                ? 'bg-aurora-cyan/10 border border-aurora-cyan/30' 
                                                : 'bg-aurora-night/30 hover:bg-aurora-night/50'
                                        }`}
                                    >
                                        <button
                                            onClick={() => handleFileSelect(i)}
                                            className={`w-5 h-5 rounded border flex items-center justify-center transition-colors flex-shrink-0 ${
                                                selectedFiles.has(i)
                                                    ? 'bg-aurora-cyan border-aurora-cyan text-aurora-void'
                                                    : 'border-aurora-border hover:border-aurora-cyan'
                                            }`}
                                        >
                                            {selectedFiles.has(i) && <Check size={14} />}
                                        </button>
                                        <File className={`w-5 h-5 flex-shrink-0 ${
                                            file.priority === 'Skip' ? 'text-aurora-dim' : 'text-aurora-muted'
                                        }`} />
                                        <div className="flex-1 min-w-0">
                                            <p className={`text-sm truncate ${
                                                file.priority === 'Skip' ? 'text-aurora-dim line-through' : 'text-aurora-text'
                                            }`}>
                                                {file.path || file.name}
                                            </p>
                                        </div>
                                        <span className="text-sm text-aurora-dim w-24 text-right">
                                            {formatSize(file.size)}
                                        </span>
                                        <div className="w-20 text-right">
                                            <span className="text-xs text-aurora-dim">
                                                {Math.round((file.progress || 0) * 100)}%
                                            </span>
                                        </div>
                                        <select
                                            value={file.priority}
                                            onChange={(e) => handleSetFilePriority(i, e.target.value as FilePriority)}
                                            className={`text-xs px-2 py-1 rounded border-0 cursor-pointer w-24 ${getPriorityColor(file.priority)}`}
                                        >
                                            <option value="High">High</option>
                                            <option value="Normal">Normal</option>
                                            <option value="Low">Low</option>
                                            <option value="Skip">Skip</option>
                                        </select>
                                    </div>
                                ))
                            ) : (
                                <p className="text-aurora-dim text-center py-8">No files</p>
                            )}
                        </div>
                    )}

                    {/* Peers Tab */}
                    {activeTab === 'peers' && (
                        <div className="space-y-2">
                            {torrent.peers?.length ? (
                                <>
                                    <div className="flex items-center gap-3 px-3 py-2 text-xs text-aurora-dim border-b border-aurora-border/30">
                                        <span className="w-40">IP</span>
                                        <span className="w-32">Client</span>
                                        <span className="w-20 text-right">Progress</span>
                                        <span className="w-24 text-right">Download</span>
                                        <span className="w-24 text-right">Upload</span>
                                        <span className="flex-1">Flags</span>
                                    </div>
                                    {torrent.peers.map((peer: PeerInfo, i: number) => (
                                        <div 
                                            key={i}
                                            className="flex items-center gap-3 p-3 rounded-lg bg-aurora-night/30"
                                        >
                                            <div className="w-40">
                                                <p className="text-sm text-aurora-text font-mono">
                                                    {peer.ip}:{peer.port}
                                                </p>
                                                {peer.country && (
                                                    <p className="text-xs text-aurora-dim">{peer.country}</p>
                                                )}
                                            </div>
                                            <div className="w-32">
                                                <p className="text-xs text-aurora-dim truncate">{peer.client || 'Unknown'}</p>
                                            </div>
                                            <div className="w-20 text-right">
                                                <div className="inline-flex items-center gap-1">
                                                    <div className="w-12 h-1.5 bg-aurora-night/50 rounded-full overflow-hidden">
                                                        <div 
                                                            className="h-full bg-aurora-cyan"
                                                            style={{ width: `${(peer.progress || 0) * 100}%` }}
                                                        />
                                                    </div>
                                                    <span className="text-xs text-aurora-dim">
                                                        {Math.round((peer.progress || 0) * 100)}%
                                                    </span>
                                                </div>
                                            </div>
                                            <div className="w-24 text-right">
                                                <span className="text-sm text-aurora-cyan">{formatSpeed(peer.down_speed)}</span>
                                            </div>
                                            <div className="w-24 text-right">
                                                <span className="text-sm text-aurora-teal">{formatSpeed(peer.up_speed)}</span>
                                            </div>
                                            <div className="flex-1">
                                                <span className="text-xs text-aurora-dim font-mono">{peer.flags || '-'}</span>
                                            </div>
                                        </div>
                                    ))}
                                </>
                            ) : (
                                <p className="text-aurora-dim text-center py-8">No peers connected</p>
                            )}
                        </div>
                    )}

                    {/* Trackers Tab */}
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
                                                : tracker.status === 'Updating'
                                                ? 'bg-amber-500'
                                                : 'bg-aurora-muted'
                                        }`} />
                                        <div className="flex-1 min-w-0">
                                            <p className="text-sm text-aurora-text font-mono truncate">
                                                {tracker.url}
                                            </p>
                                            {tracker.message && (
                                                <p className="text-xs text-aurora-dim mt-1">{tracker.message}</p>
                                            )}
                                        </div>
                                        <div className="text-right text-xs text-aurora-dim">
                                            <p>Seeds: {tracker.seeds} | Leechers: {tracker.leechers}</p>
                                            {tracker.next_announce && (
                                                <p>Next: {formatDuration(tracker.next_announce)}</p>
                                            )}
                                        </div>
                                        <span className={`text-xs px-2 py-1 rounded ${
                                            tracker.status === 'Working' || tracker.status === 'Active'
                                                ? 'bg-aurora-teal/10 text-aurora-teal'
                                                : tracker.status === 'Updating'
                                                ? 'bg-amber-500/10 text-amber-500'
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

                    {/* Speed Tab */}
                    {activeTab === 'speed' && (
                        <div className="space-y-6">
                            <SpeedGraph 
                                samples={speedSamples}
                                height={200}
                                showLegend={true}
                            />
                            
                            <div className="grid grid-cols-2 gap-4">
                                <InfoCard label="Current Download" value={formatSpeed(torrent.download_speed)} />
                                <InfoCard label="Current Upload" value={formatSpeed(torrent.upload_speed)} />
                                <InfoCard label="Downloaded" value={formatSize(torrent.downloaded)} />
                                <InfoCard label="Uploaded" value={formatSize(torrent.uploaded)} />
                            </div>
                        </div>
                    )}
                </div>

                {/* Confirm remove dialog */}
                <AnimatePresence>
                    {showConfirmRemove && (
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            className="absolute inset-0 bg-aurora-void/90 backdrop-blur-sm flex items-center justify-center p-6"
                        >
                            <div className="card p-6 max-w-sm w-full">
                                <Trash2 className="w-12 h-12 text-aurora-rose mx-auto mb-4" />
                                <h3 className="text-lg font-bold mb-2 text-center">Remove Torrent?</h3>
                                <p className="text-aurora-dim text-sm mb-4 text-center">
                                    This will remove "{torrent.name}" from the list.
                                </p>
                                
                                <label className="flex items-center gap-3 p-3 rounded-lg bg-aurora-night/30 mb-6 cursor-pointer hover:bg-aurora-night/50 transition-colors">
                                    <input
                                        type="checkbox"
                                        checked={deleteFilesOnRemove}
                                        onChange={(e) => setDeleteFilesOnRemove(e.target.checked)}
                                        className="w-4 h-4 rounded border-aurora-border accent-aurora-rose"
                                    />
                                    <span className="text-sm text-aurora-text">
                                        Also delete downloaded files
                                    </span>
                                </label>

                                <div className="flex gap-3 justify-center">
                                    <button
                                        onClick={() => {
                                            setShowConfirmRemove(false);
                                            setDeleteFilesOnRemove(false);
                                        }}
                                        className="btn-secondary"
                                    >
                                        Cancel
                                    </button>
                                    <button
                                        onClick={() => {
                                            onRemove(torrent.id, deleteFilesOnRemove);
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
                </AnimatePresence>
            </motion.div>
        </motion.div>
    );
}

function InfoCard({ label, value }: { label: string; value: string }) {
    return (
        <div className="p-4 rounded-lg bg-aurora-night/30">
            <p className="text-xs text-aurora-dim mb-1">{label}</p>
            <p className="text-sm font-medium text-aurora-text">{value}</p>
        </div>
    );
}
