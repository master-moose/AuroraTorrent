import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
    Download, Upload, Users, HardDrive, Wifi, WifiOff,
    Zap, ZapOff, AlertCircle, Activity
} from 'lucide-react';
import { SessionStats } from '../types';

interface StatusBarProps {
    stats: SessionStats;
    isConnected: boolean;
    altSpeedEnabled: boolean;
    onToggleAltSpeed: () => void;
    className?: string;
}

export default function StatusBar({
    stats,
    isConnected,
    altSpeedEnabled,
    onToggleAltSpeed,
    className = ''
}: StatusBarProps) {
    const [showDetails, setShowDetails] = useState(false);

    const formatSpeed = (bytes: number) => {
        if (bytes === 0) return '0 B/s';
        if (bytes < 1024) return `${bytes} B/s`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
    };

    const formatSize = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    const formatUptime = (seconds: number) => {
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
        if (seconds < 86400) {
            const hours = Math.floor(seconds / 3600);
            const mins = Math.floor((seconds % 3600) / 60);
            return `${hours}h ${mins}m`;
        }
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        return `${days}d ${hours}h`;
    };

    return (
        <>
            <div 
                className={`h-8 bg-aurora-night/50 border-t border-aurora-border/30 flex items-center px-4 text-xs gap-6 ${className}`}
            >
                {/* Connection Status */}
                <div className="flex items-center gap-2">
                    {isConnected ? (
                        <>
                            <Wifi size={14} className="text-aurora-teal" />
                            <span className="text-aurora-dim">Connected</span>
                        </>
                    ) : (
                        <>
                            <WifiOff size={14} className="text-aurora-rose" />
                            <span className="text-aurora-rose">Disconnected</span>
                        </>
                    )}
                </div>

                <div className="h-4 w-px bg-aurora-border/30" />

                {/* Download Speed */}
                <div className="flex items-center gap-2">
                    <Download size={14} className="text-aurora-cyan" />
                    <span className="text-aurora-cyan font-medium">
                        {formatSpeed(stats.download_rate)}
                    </span>
                </div>

                {/* Upload Speed */}
                <div className="flex items-center gap-2">
                    <Upload size={14} className="text-aurora-teal" />
                    <span className="text-aurora-teal font-medium">
                        {formatSpeed(stats.upload_rate)}
                    </span>
                </div>

                <div className="h-4 w-px bg-aurora-border/30" />

                {/* Alt Speed Toggle */}
                <button
                    onClick={onToggleAltSpeed}
                    className={`flex items-center gap-1.5 px-2 py-1 rounded transition-colors ${
                        altSpeedEnabled 
                            ? 'bg-aurora-violet/20 text-aurora-violet' 
                            : 'text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50'
                    }`}
                    title={altSpeedEnabled ? 'Alternative speed limits active' : 'Enable alternative speed limits'}
                >
                    {altSpeedEnabled ? <Zap size={14} /> : <ZapOff size={14} />}
                    <span>Alt Speed</span>
                </button>

                <div className="h-4 w-px bg-aurora-border/30" />

                {/* DHT Nodes */}
                <div className="flex items-center gap-2 text-aurora-dim">
                    <Users size={14} />
                    <span>{stats.dht_nodes} DHT</span>
                </div>

                {/* Peers */}
                <div className="flex items-center gap-2 text-aurora-dim">
                    <Activity size={14} />
                    <span>{stats.peers_connected} peers</span>
                </div>

                {/* Spacer */}
                <div className="flex-1" />

                {/* Torrent Counts */}
                <div className="flex items-center gap-4 text-aurora-dim">
                    {stats.downloading_torrents > 0 && (
                        <span className="flex items-center gap-1">
                            <Download size={12} className="text-aurora-cyan" />
                            {stats.downloading_torrents}
                        </span>
                    )}
                    {stats.seeding_torrents > 0 && (
                        <span className="flex items-center gap-1">
                            <Upload size={12} className="text-aurora-teal" />
                            {stats.seeding_torrents}
                        </span>
                    )}
                    {stats.paused_torrents > 0 && (
                        <span className="flex items-center gap-1 opacity-50">
                            ⏸ {stats.paused_torrents}
                        </span>
                    )}
                    {stats.error_torrents > 0 && (
                        <span className="flex items-center gap-1 text-aurora-rose">
                            <AlertCircle size={12} />
                            {stats.error_torrents}
                        </span>
                    )}
                </div>

                <div className="h-4 w-px bg-aurora-border/30" />

                {/* Global Ratio */}
                <div className="flex items-center gap-2 text-aurora-dim">
                    <span>Ratio:</span>
                    <span className={stats.global_ratio >= 1 ? 'text-aurora-teal' : 'text-aurora-dim'}>
                        {stats.global_ratio.toFixed(2)}
                    </span>
                </div>

                {/* More Details Button */}
                <button
                    onClick={() => setShowDetails(true)}
                    className="text-aurora-dim hover:text-aurora-cyan transition-colors"
                    title="Show statistics"
                >
                    •••
                </button>
            </div>

            {/* Statistics Popup */}
            <AnimatePresence>
                {showDetails && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 bg-aurora-void/80 backdrop-blur-sm z-[100] flex items-center justify-center p-4"
                        onClick={() => setShowDetails(false)}
                    >
                        <motion.div
                            initial={{ opacity: 0, scale: 0.95, y: 20 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.95, y: 20 }}
                            className="card w-full max-w-md"
                            onClick={e => e.stopPropagation()}
                        >
                            <div className="p-6 border-b border-aurora-border/50">
                                <h2 className="text-xl font-bold text-aurora-text flex items-center gap-2">
                                    <Activity size={20} className="text-aurora-cyan" />
                                    Session Statistics
                                </h2>
                            </div>
                            
                            <div className="p-6 space-y-6">
                                {/* Transfer Stats */}
                                <div>
                                    <h3 className="text-sm font-medium text-aurora-dim mb-3">Transfer</h3>
                                    <div className="grid grid-cols-2 gap-4">
                                        <StatItem
                                            label="Downloaded (session)"
                                            value={formatSize(stats.total_downloaded_session)}
                                            icon={<Download size={16} className="text-aurora-cyan" />}
                                        />
                                        <StatItem
                                            label="Uploaded (session)"
                                            value={formatSize(stats.total_uploaded_session)}
                                            icon={<Upload size={16} className="text-aurora-teal" />}
                                        />
                                        <StatItem
                                            label="Downloaded (all-time)"
                                            value={formatSize(stats.total_downloaded)}
                                        />
                                        <StatItem
                                            label="Uploaded (all-time)"
                                            value={formatSize(stats.total_uploaded)}
                                        />
                                    </div>
                                </div>

                                {/* Performance */}
                                <div>
                                    <h3 className="text-sm font-medium text-aurora-dim mb-3">Performance</h3>
                                    <div className="grid grid-cols-2 gap-4">
                                        <StatItem
                                            label="Disk Read Rate"
                                            value={formatSpeed(stats.disk_read_rate)}
                                            icon={<HardDrive size={16} className="text-aurora-dim" />}
                                        />
                                        <StatItem
                                            label="Disk Write Rate"
                                            value={formatSpeed(stats.disk_write_rate)}
                                        />
                                        <StatItem
                                            label="Cache Size"
                                            value={formatSize(stats.disk_cache_size)}
                                        />
                                        <StatItem
                                            label="Cache Usage"
                                            value={`${(stats.disk_cache_usage * 100).toFixed(0)}%`}
                                        />
                                    </div>
                                </div>

                                {/* Network */}
                                <div>
                                    <h3 className="text-sm font-medium text-aurora-dim mb-3">Network</h3>
                                    <div className="grid grid-cols-2 gap-4">
                                        <StatItem
                                            label="DHT Nodes"
                                            value={stats.dht_nodes.toString()}
                                            icon={<Users size={16} className="text-aurora-dim" />}
                                        />
                                        <StatItem
                                            label="Connected Peers"
                                            value={stats.peers_connected.toString()}
                                        />
                                        <StatItem
                                            label="Global Ratio"
                                            value={stats.global_ratio.toFixed(3)}
                                        />
                                        <StatItem
                                            label="Wasted"
                                            value={formatSize(stats.total_wasted)}
                                        />
                                    </div>
                                </div>

                                {/* Session Info */}
                                <div>
                                    <h3 className="text-sm font-medium text-aurora-dim mb-3">Session</h3>
                                    <div className="grid grid-cols-2 gap-4">
                                        <StatItem
                                            label="Uptime"
                                            value={formatUptime(stats.up_time)}
                                        />
                                        <StatItem
                                            label="Total Torrents"
                                            value={stats.total_torrents.toString()}
                                        />
                                    </div>
                                </div>
                            </div>

                            <div className="flex justify-end p-6 border-t border-aurora-border/50">
                                <button 
                                    onClick={() => setShowDetails(false)}
                                    className="btn-secondary"
                                >
                                    Close
                                </button>
                            </div>
                        </motion.div>
                    </motion.div>
                )}
            </AnimatePresence>
        </>
    );
}

function StatItem({ 
    label, 
    value, 
    icon 
}: { 
    label: string; 
    value: string; 
    icon?: React.ReactNode;
}) {
    return (
        <div className="p-3 rounded-lg bg-aurora-night/30">
            <div className="flex items-center gap-2 mb-1">
                {icon}
                <span className="text-xs text-aurora-dim">{label}</span>
            </div>
            <span className="text-sm font-medium text-aurora-text">{value}</span>
        </div>
    );
}

// Default stats for when not connected
export const defaultSessionStats: SessionStats = {
    total_downloaded: 0,
    total_uploaded: 0,
    total_wasted: 0,
    total_downloaded_session: 0,
    total_uploaded_session: 0,
    total_torrents: 0,
    downloading_torrents: 0,
    seeding_torrents: 0,
    paused_torrents: 0,
    checking_torrents: 0,
    error_torrents: 0,
    global_ratio: 0,
    dht_nodes: 0,
    peers_connected: 0,
    download_rate: 0,
    upload_rate: 0,
    disk_read_rate: 0,
    disk_write_rate: 0,
    disk_cache_size: 0,
    disk_cache_usage: 0,
    up_time: 0
};

