import { motion } from 'framer-motion';
import { Play, Pause, Download, CheckCircle, Clock, Plus } from 'lucide-react';
import { Torrent } from '../types';

interface LibraryGridProps {
    torrents: Torrent[];
    onStream?: (id: string) => void;
    onSelect?: (t: Torrent) => void;
    onAdd?: () => void;
    compact?: boolean;
}

export default function LibraryGrid({ torrents, onStream, onSelect, onAdd, compact }: LibraryGridProps) {
    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'Downloading':
            case 'Streaming':
                return <Download className="w-4 h-4 animate-pulse" />;
            case 'Seeding':
                return <CheckCircle className="w-4 h-4" />;
            case 'Paused':
                return <Pause className="w-4 h-4" />;
            default:
                return <Clock className="w-4 h-4" />;
        }
    };

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'Downloading':
            case 'Streaming':
                return 'text-aurora-cyan';
            case 'Seeding':
                return 'text-aurora-teal';
            case 'Paused':
                return 'text-aurora-dim';
            default:
                return 'text-aurora-muted';
        }
    };

    const getGradient = (index: number) => {
        const gradients = [
            'from-aurora-cyan/80 to-aurora-teal/60',
            'from-aurora-violet/80 to-aurora-magenta/60',
            'from-aurora-magenta/80 to-aurora-rose/60',
            'from-aurora-teal/80 to-aurora-cyan/60',
            'from-aurora-rose/80 to-aurora-violet/60',
        ];
        return gradients[index % gradients.length];
    };

    const formatSize = (bytes: number) => {
        if (bytes === 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
    };

    const container = {
        hidden: { opacity: 0 },
        show: {
            opacity: 1,
            transition: { staggerChildren: 0.05 }
        }
    };

    const item = {
        hidden: { opacity: 0, y: 20 },
        show: { opacity: 1, y: 0 }
    };

    return (
        <motion.div 
            className={`grid gap-4 ${
                compact 
                    ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5'
                    : 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5'
            }`}
            variants={container}
            initial="hidden"
            animate="show"
        >
            {/* Add New Card */}
            {onAdd && !compact && (
                <motion.div
                    variants={item}
                    onClick={onAdd}
                    className="card card-hover p-4 cursor-pointer group"
                >
                    <div className="aspect-square rounded-lg bg-aurora-night/50 border-2 border-dashed border-aurora-border flex flex-col items-center justify-center mb-4 group-hover:border-aurora-cyan/50 transition-colors">
                        <Plus className="w-12 h-12 text-aurora-muted group-hover:text-aurora-cyan transition-colors mb-2" />
                        <span className="text-sm text-aurora-muted group-hover:text-aurora-dim transition-colors">
                            Add Torrent
                        </span>
                    </div>
                    <h3 className="font-semibold text-aurora-text">Add New</h3>
                    <p className="text-sm text-aurora-dim">Magnet or .torrent</p>
                </motion.div>
            )}

            {/* Torrent Cards */}
            {torrents.map((torrent, index) => (
                <motion.div
                    key={torrent.id}
                    variants={item}
                    onClick={() => onSelect?.(torrent)}
                    className="card card-hover p-4 cursor-pointer group relative"
                >
                    {/* Thumbnail */}
                    <div className={`aspect-square rounded-lg bg-gradient-to-br ${getGradient(index)} mb-4 relative overflow-hidden`}>
                        {/* Progress overlay */}
                        {torrent.progress < 1 && (
                            <div 
                                className="absolute bottom-0 left-0 right-0 h-1 bg-aurora-void/50"
                            >
                                <motion.div 
                                    className="h-full bg-white/80"
                                    initial={{ width: 0 }}
                                    animate={{ width: `${torrent.progress * 100}%` }}
                                    transition={{ duration: 0.3 }}
                                />
                            </div>
                        )}

                        {/* Play button */}
                        <motion.button
                            onClick={(e) => {
                                e.stopPropagation();
                                onStream?.(torrent.id);
                            }}
                            className="absolute bottom-3 right-3 w-12 h-12 bg-aurora-cyan rounded-full flex items-center justify-center shadow-lg opacity-0 translate-y-2 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-200 hover:bg-aurora-teal hover:scale-105"
                            whileTap={{ scale: 0.95 }}
                        >
                            <Play className="w-5 h-5 text-aurora-void ml-0.5" fill="currentColor" />
                        </motion.button>

                        {/* File type icon */}
                        <div className="absolute top-3 left-3 px-2 py-1 bg-black/30 backdrop-blur-sm rounded text-xs font-medium">
                            {torrent.files?.[0]?.name?.split('.').pop()?.toUpperCase() || 'FILE'}
                        </div>
                    </div>

                    {/* Info */}
                    <h3 className="font-semibold text-aurora-text truncate mb-1" title={torrent.name}>
                        {torrent.name}
                    </h3>
                    
                    <div className="flex items-center gap-2 text-sm">
                        <span className={`flex items-center gap-1 ${getStatusColor(torrent.status)}`}>
                            {getStatusIcon(torrent.status)}
                            {torrent.status}
                        </span>
                        <span className="text-aurora-muted">•</span>
                        <span className="text-aurora-dim">
                            {Math.round(torrent.progress * 100)}%
                        </span>
                    </div>

                    {!compact && (
                        <p className="text-xs text-aurora-muted mt-2">
                            {formatSize(torrent.total_size)}
                        </p>
                    )}
                </motion.div>
            ))}
        </motion.div>
    );
}
