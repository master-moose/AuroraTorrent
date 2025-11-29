import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import Sidebar from './components/Sidebar';
import LibraryGrid from './components/LibraryGrid';
import NowPlayingFooter from './components/NowPlayingFooter';
import VideoPlayer from './components/VideoPlayer';
import SettingsModal from './components/SettingsModal';
import TorrentDetails from './components/TorrentDetails';
import Toast from './components/Toast';
import AddTorrentModal from './components/AddTorrentModal';
import ErrorBoundary from './components/ErrorBoundary';
import FilterSidebar, { TorrentFilter, filterTorrents, defaultFilter } from './components/FilterSidebar';
import StatusBar, { defaultSessionStats } from './components/StatusBar';
import SpeedGraph, { useSpeedSamples } from './components/SpeedGraph';
import {
    listTorrents,
    addTorrent as rpcAddTorrent,
    addTorrentFile as rpcAddTorrentFile,
    startTorrent,
    pauseTorrent,
    removeTorrent,
    streamTorrent,
    getConfig,
    toggleAltSpeed,
} from './rpc';
import { Torrent, SessionStats, Category, Config } from './types';

export interface ToastMessage {
    id: string;
    type: 'success' | 'error' | 'info';
    message: string;
}

function App() {
    const [view, setView] = useState<'home' | 'library' | 'search' | 'rss'>('home');
    const [torrents, setTorrents] = useState<Torrent[]>([]);
    const [activeStream, setActiveStream] = useState<{
        url: string;
        torrentId: string;
        fileIndex: number;
        fileName?: string;
    } | null>(null);
    const [showSettings, setShowSettings] = useState(false);
    const [showAddTorrent, setShowAddTorrent] = useState(false);
    const [selectedTorrent, setSelectedTorrent] = useState<Torrent | null>(null);
    const [toasts, setToasts] = useState<ToastMessage[]>([]);
    const [isDragging, setIsDragging] = useState(false);
    const [showFilters, setShowFilters] = useState(true);
    const [filter, setFilter] = useState<TorrentFilter>(defaultFilter);
    const [config, setConfig] = useState<Partial<Config>>({});
    const [altSpeedEnabled, setAltSpeedEnabled] = useState(false);

    const addToast = useCallback((type: ToastMessage['type'], message: string) => {
        const id = Date.now().toString();
        setToasts(prev => [...prev, { id, type, message }]);
        setTimeout(() => {
            setToasts(prev => prev.filter(t => t.id !== id));
        }, 4000);
    }, []);

    const removeToast = useCallback((id: string) => {
        setToasts(prev => prev.filter(t => t.id !== id));
    }, []);

    // Track mounted state to prevent state updates after unmount
    const isMountedRef = useRef(true);

    useEffect(() => {
        isMountedRef.current = true;

        const fetchTorrents = async () => {
            try {
                const torrents = await listTorrents();
                if (isMountedRef.current) {
                    setTorrents(torrents);
                }
            } catch (e) {
                console.error('Failed to fetch torrents:', e);
            }
        };

        const fetchConfig = async () => {
            try {
                const cfg = await getConfig();
                if (cfg && isMountedRef.current) {
                    setConfig(cfg);
                    setAltSpeedEnabled(cfg.use_alt_speed_limits || false);
                }
            } catch (e) {
                console.error('Failed to fetch config:', e);
            }
        };

        fetchTorrents();
        fetchConfig();
        const interval = setInterval(fetchTorrents, 1000);

        return () => {
            isMountedRef.current = false;
            clearInterval(interval);
        };
    }, []);

    // Calculate session stats from torrents
    const sessionStats: SessionStats = useMemo(() => {
        const stats = { ...defaultSessionStats };
        stats.total_torrents = torrents.length;
        
        for (const t of torrents) {
            stats.download_rate += t.download_speed;
            stats.upload_rate += t.upload_speed;
            stats.total_downloaded_session += t.downloaded_session;
            stats.total_uploaded_session += t.uploaded_session;
            stats.total_downloaded += t.downloaded;
            stats.total_uploaded += t.uploaded;
            stats.peers_connected += t.connected_seeds + t.connected_leechers;
            
            const status = t.status.toLowerCase();
            if (status === 'downloading' || status === 'forceddownloading') {
                stats.downloading_torrents++;
            } else if (status === 'seeding' || status === 'forcedseeding') {
                stats.seeding_torrents++;
            } else if (status === 'paused') {
                stats.paused_torrents++;
            } else if (status === 'checking') {
                stats.checking_torrents++;
            } else if (status === 'error') {
                stats.error_torrents++;
            }
        }
        
        if (stats.total_downloaded > 0) {
            stats.global_ratio = stats.total_uploaded / stats.total_downloaded;
        }
        
        return stats;
    }, [torrents]);

    // Speed samples for graphing
    const speedSamples = useSpeedSamples(sessionStats.download_rate, sessionStats.upload_rate);

    // Filtered torrents
    const filteredTorrents = useMemo(() => {
        return filterTorrents(torrents, filter);
    }, [torrents, filter]);

    // Categories and tags from config
    const categories = (config.categories || {}) as Record<string, Category>;
    const tags = (config.tags || []) as string[];

    const handleStreamStart = async (id: string, fileIndex: number = 0) => {
        const result = await streamTorrent(id);
        const torrent = torrents.find(t => t.id === id);
        if (result.url) {
            setActiveStream({
                url: result.url,
                torrentId: id,
                fileIndex,
                fileName: torrent?.name,
            });
            addToast('info', 'Starting stream...');
        } else {
            addToast('error', result.error || 'Failed to start stream');
        }
    };

    const handleAddTorrent = async (magnet: string) => {
        const result = await rpcAddTorrent(magnet);
        if (!result.error) {
            addToast('success', `Added: ${result.name || 'New torrent'}`);
            setShowAddTorrent(false);
        } else {
            addToast('error', result.error || 'Failed to add torrent');
        }
    };

    const handleAddTorrentFile = async (file: File) => {
        const reader = new FileReader();
        reader.onload = async (e) => {
            const content = e.target?.result as string;
            const result = await rpcAddTorrentFile(
                file.name,
                content.split(',')[1] || content
            );
            if (!result.error) {
                addToast('success', `Added: ${result.name || file.name}`);
                setShowAddTorrent(false);
            } else {
                addToast('error', result.error || 'Failed to add torrent file');
            }
        };
        reader.readAsDataURL(file);
    };

    const handlePauseTorrent = async (id: string) => {
        const result = await pauseTorrent(id);
        if (!result.error) {
            addToast('info', 'Torrent paused');
        }
    };

    const handleResumeTorrent = async (id: string) => {
        const result = await startTorrent(id);
        if (!result.error) {
            addToast('info', 'Torrent resumed');
        }
    };

    const handleRemoveTorrent = async (id: string, deleteFiles: boolean = false) => {
        const result = await removeTorrent(id, deleteFiles);
        if (!result.error) {
            addToast('success', deleteFiles ? 'Torrent and files removed' : 'Torrent removed');
            setSelectedTorrent(null);
        } else {
            addToast('error', result.error || 'Failed to remove torrent');
        }
    };

    const handleRefreshTorrents = useCallback(async () => {
        try {
            const updatedTorrents = await listTorrents();
            if (isMountedRef.current) {
                setTorrents(updatedTorrents);
                // Update selected torrent if still open
                if (selectedTorrent) {
                    const updated = updatedTorrents.find(t => t.id === selectedTorrent.id);
                    if (updated) {
                        setSelectedTorrent(updated);
                    }
                }
            }
        } catch (e) {
            console.error('Failed to refresh torrents:', e);
        }
    }, [selectedTorrent]);

    const handleToggleAltSpeed = async () => {
        const newEnabled = !altSpeedEnabled;
        setAltSpeedEnabled(newEnabled);
        
        const result = await toggleAltSpeed(newEnabled);
        if (result.error) {
             addToast('error', result.error);
             // Revert state on error
             setAltSpeedEnabled(altSpeedEnabled);
        } else {
             addToast('info', newEnabled ? 'Alternative speed limits enabled' : 'Alternative speed limits disabled');
        }
    };

    // Drag and drop handlers
    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(true);
    };

    const handleDragLeave = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
    };

    const handleDrop = async (e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);

        const files = Array.from(e.dataTransfer.files);
        const torrentFile = files.find(f => f.name.endsWith('.torrent'));

        if (torrentFile) {
            await handleAddTorrentFile(torrentFile);
        } else {
            const text = e.dataTransfer.getData('text');
            if (text.startsWith('magnet:')) {
                await handleAddTorrent(text);
            }
        }
    };

    const activeDownloads = torrents.filter(t => t.status === 'Downloading').length;
    const activeTorrent = torrents.find(t => 
        t.status === 'Downloading' || t.status === 'Seeding'
    ) || torrents[0];

    return (
        <ErrorBoundary>
        <div 
            className="flex flex-col h-screen w-screen overflow-hidden relative"
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
        >
            {/* Aurora background effect */}
            <div className="fixed inset-0 pointer-events-none">
                <div className="absolute inset-0 bg-aurora-gradient opacity-50" />
                <div className="absolute top-0 left-1/4 w-96 h-96 bg-aurora-cyan/5 rounded-full blur-[100px]" />
                <div className="absolute top-1/4 right-1/4 w-80 h-80 bg-aurora-violet/5 rounded-full blur-[100px]" />
            </div>

            {/* Drag overlay */}
            <AnimatePresence>
                {isDragging && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-[200] bg-aurora-void/90 backdrop-blur-sm flex items-center justify-center"
                    >
                        <div className="card aurora-border p-12 text-center">
                            <div className="text-6xl mb-4">📥</div>
                            <h2 className="text-2xl font-bold text-aurora-text mb-2">Drop to Add</h2>
                            <p className="text-aurora-dim">Release to add .torrent file or magnet link</p>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Modals */}
            <AnimatePresence>
                {activeStream && (
                    <VideoPlayer
                        streamUrl={activeStream.url}
                        torrentId={activeStream.torrentId}
                        fileIndex={activeStream.fileIndex}
                        fileName={activeStream.fileName}
                        onClose={() => setActiveStream(null)}
                    />
                )}
                {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
                {showAddTorrent && (
                    <AddTorrentModal 
                        onClose={() => setShowAddTorrent(false)}
                        onAddMagnet={handleAddTorrent}
                        onAddFile={handleAddTorrentFile}
                    />
                )}
                {selectedTorrent && (
                    <TorrentDetails 
                        torrent={selectedTorrent} 
                        onClose={() => setSelectedTorrent(null)}
                        onPause={handlePauseTorrent}
                        onResume={handleResumeTorrent}
                        onRemove={handleRemoveTorrent}
                        onStream={handleStreamStart}
                        onRefresh={handleRefreshTorrents}
                    />
                )}
            </AnimatePresence>

            {/* Toast notifications */}
            <div className="fixed top-4 right-4 z-[150] flex flex-col gap-2">
                <AnimatePresence>
                    {toasts.map(toast => (
                        <Toast key={toast.id} {...toast} onClose={() => removeToast(toast.id)} />
                    ))}
                </AnimatePresence>
            </div>

            {/* Main layout */}
            <div className="flex flex-1 overflow-hidden relative z-10">
                <Sidebar 
                    currentView={view} 
                    setView={setView} 
                    activeDownloads={activeDownloads}
                    onOpenSettings={() => setShowSettings(true)}
                    showFilters={showFilters}
                    onToggleFilters={() => setShowFilters(!showFilters)}
                />

                {/* Filter Sidebar */}
                <AnimatePresence>
                    {showFilters && view === 'library' && (
                        <motion.div
                            initial={{ width: 0, opacity: 0 }}
                            animate={{ width: 'auto', opacity: 1 }}
                            exit={{ width: 0, opacity: 0 }}
                            transition={{ duration: 0.2 }}
                            className="border-r border-aurora-border/30 bg-aurora-void/30"
                        >
                            <FilterSidebar
                                torrents={torrents}
                                categories={categories}
                                tags={tags}
                                selectedFilter={filter}
                                onFilterChange={setFilter}
                            />
                        </motion.div>
                    )}
                </AnimatePresence>

                <main className="flex-1 overflow-y-auto p-6">
                    <header className="flex justify-between items-center mb-6">
                        <div>
                            <h1 className="text-3xl font-bold text-aurora-text">
                                {view === 'home' ? 'Welcome Back' : 
                                 view === 'library' ? 'Your Library' : 
                                 view === 'rss' ? 'RSS Feeds' : 'Search'}
                            </h1>
                            <p className="text-aurora-dim mt-1">
                                {view === 'library' && filter.status !== 'all' 
                                    ? `${filteredTorrents.length} of ${torrents.length} torrents • ${activeDownloads} active`
                                    : `${torrents.length} torrents • ${activeDownloads} active`
                                }
                            </p>
                        </div>
                        <button 
                            onClick={() => setShowAddTorrent(true)}
                            className="btn-primary flex items-center gap-2"
                        >
                            <span className="text-lg">+</span>
                            Add Torrent
                        </button>
                    </header>

                    {view === 'home' && (
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="space-y-8"
                        >
                            {/* Speed Graph */}
                            {speedSamples.length > 1 && (
                                <SpeedGraph 
                                    samples={speedSamples} 
                                    height={140}
                                    className="mb-6"
                                />
                            )}

                            {/* Quick stats */}
                            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Active Downloads</div>
                                    <div className="text-3xl font-bold text-aurora-cyan">{sessionStats.downloading_torrents}</div>
                                </div>
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Download Speed</div>
                                    <div className="text-3xl font-bold text-aurora-teal">
                                        {formatSpeed(sessionStats.download_rate)}
                                    </div>
                                </div>
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Upload Speed</div>
                                    <div className="text-3xl font-bold text-aurora-violet">
                                        {formatSpeed(sessionStats.upload_rate)}
                                    </div>
                                </div>
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Share Ratio</div>
                                    <div className={`text-3xl font-bold ${
                                        sessionStats.global_ratio >= 1 ? 'text-aurora-teal' : 'text-aurora-dim'
                                    }`}>
                                        {sessionStats.global_ratio.toFixed(2)}
                                    </div>
                                </div>
                            </div>

                            {/* Recent/Active torrents */}
                            {torrents.length > 0 ? (
                                <div>
                                    <h2 className="text-xl font-semibold mb-4">Recent Activity</h2>
                                    <LibraryGrid 
                                        torrents={torrents.slice(0, 6)} 
                                        onStream={handleStreamStart}
                                        onSelect={setSelectedTorrent}
                                        compact
                                    />
                                </div>
                            ) : (
                                <div className="card p-12 text-center">
                                    <div className="text-6xl mb-4 opacity-50">🌌</div>
                                    <h2 className="text-xl font-semibold mb-2">No torrents yet</h2>
                                    <p className="text-aurora-dim mb-6">
                                        Add a magnet link or .torrent file to get started
                                    </p>
                                    <button 
                                        onClick={() => setShowAddTorrent(true)}
                                        className="btn-primary"
                                    >
                                        Add Your First Torrent
                                    </button>
                                </div>
                            )}
                        </motion.div>
                    )}

                    {view === 'library' && (
                        <LibraryGrid 
                            torrents={filteredTorrents} 
                            onStream={handleStreamStart}
                            onSelect={setSelectedTorrent}
                            onAdd={() => setShowAddTorrent(true)}
                        />
                    )}

                    {view === 'search' && (
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="text-center py-12"
                        >
                            <div className="text-6xl mb-4 opacity-50">🔍</div>
                            <h2 className="text-xl font-semibold mb-2">Search Coming Soon</h2>
                            <p className="text-aurora-dim">
                                Search through your torrents and discover new content with search plugins
                            </p>
                        </motion.div>
                    )}

                    {view === 'rss' && (
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="text-center py-12"
                        >
                            <div className="text-6xl mb-4 opacity-50">📡</div>
                            <h2 className="text-xl font-semibold mb-2">RSS Feeds Coming Soon</h2>
                            <p className="text-aurora-dim">
                                Subscribe to RSS feeds and automatically download new torrents
                            </p>
                        </motion.div>
                    )}
                </main>
            </div>

            {/* Now Playing Footer (for streaming torrents) */}
            {activeTorrent && view !== 'library' && (
                <NowPlayingFooter 
                    torrent={activeTorrent} 
                    onStream={activeTorrent ? () => handleStreamStart(activeTorrent.id) : undefined}
                />
            )}

            {/* Status Bar */}
            <StatusBar
                stats={sessionStats}
                isConnected={true}
                altSpeedEnabled={altSpeedEnabled}
                onToggleAltSpeed={handleToggleAltSpeed}
            />
        </div>
        </ErrorBoundary>
    );
}

function formatSpeed(bytes: number): string {
    if (bytes === 0) return '0 B/s';
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export default App;
