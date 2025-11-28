import { Play, Download, ArrowUp, ArrowDown } from 'lucide-react';
import { motion } from 'framer-motion';
import { Torrent } from '../types';

interface NowPlayingFooterProps {
    torrent?: Torrent;
    onStream?: () => void;
}

export default function NowPlayingFooter({ torrent, onStream }: NowPlayingFooterProps) {
    const formatSpeed = (bytes: number) => {
        if (bytes === 0) return '0 B/s';
        const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    const formatSize = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    return (
        <div className="h-20 bg-aurora-deep/80 backdrop-blur-md border-t border-aurora-border/30 px-6 flex items-center justify-between relative z-50">
            {/* Left: Current torrent info */}
            <div className="flex items-center gap-4 w-[30%] min-w-0">
                {torrent ? (
                    <>
                        <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-aurora-cyan to-aurora-violet flex-shrink-0" />
                        <div className="min-w-0">
                            <p className="font-medium text-aurora-text truncate text-sm">
                                {torrent.name}
                            </p>
                            <p className="text-xs text-aurora-dim">
                                {torrent.status} • {formatSize(torrent.total_size)}
                            </p>
                        </div>
                    </>
                ) : (
                    <p className="text-sm text-aurora-dim">No active torrents</p>
                )}
            </div>

            {/* Center: Progress and controls */}
            <div className="flex flex-col items-center w-[40%] gap-2">
                {torrent && (
                    <>
                        {/* Play/stream button */}
                        {torrent.status === 'Downloading' || torrent.status === 'Streaming' ? (
                            <motion.button
                                onClick={onStream}
                                className="w-10 h-10 bg-aurora-cyan rounded-full flex items-center justify-center hover:bg-aurora-teal transition-colors"
                                whileHover={{ scale: 1.05 }}
                                whileTap={{ scale: 0.95 }}
                            >
                                <Play className="w-5 h-5 text-aurora-void ml-0.5" fill="currentColor" />
                            </motion.button>
                        ) : (
                            <div className="w-10 h-10 bg-aurora-night rounded-full flex items-center justify-center">
                                <Download className="w-5 h-5 text-aurora-dim" />
                            </div>
                        )}

                        {/* Progress bar */}
                        <div className="w-full flex items-center gap-3">
                            <span className="text-xs text-aurora-dim font-mono w-12 text-right">
                                {Math.round(torrent.progress * 100)}%
                            </span>
                            <div className="flex-1 progress-bar">
                                <motion.div 
                                    className="progress-bar-fill"
                                    initial={{ width: 0 }}
                                    animate={{ width: `${torrent.progress * 100}%` }}
                                    transition={{ duration: 0.3, ease: 'easeOut' }}
                                />
                            </div>
                            <span className="text-xs text-aurora-dim font-mono w-16">
                                {formatSize(torrent.total_size)}
                            </span>
                        </div>
                    </>
                )}
            </div>

            {/* Right: Speed indicators */}
            <div className="flex items-center justify-end gap-6 w-[30%]">
                {torrent && (
                    <>
                        <div className="flex items-center gap-2">
                            <ArrowDown className="w-4 h-4 text-aurora-cyan" />
                            <div>
                                <p className="text-sm font-mono text-aurora-text">
                                    {formatSpeed(torrent.download_speed)}
                                </p>
                                <p className="text-xs text-aurora-dim">Download</p>
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <ArrowUp className="w-4 h-4 text-aurora-teal" />
                            <div>
                                <p className="text-sm font-mono text-aurora-text">
                                    {formatSpeed(torrent.upload_speed)}
                                </p>
                                <p className="text-xs text-aurora-dim">Upload</p>
                            </div>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
}
