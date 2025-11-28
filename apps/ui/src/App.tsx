import { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import Sidebar from './components/Sidebar';
import LibraryGrid from './components/LibraryGrid';
import NowPlayingFooter from './components/NowPlayingFooter';
import VideoPlayer from './components/VideoPlayer';
import SettingsModal from './components/SettingsModal';
import TorrentDetails from './components/TorrentDetails';
import Toast from './components/Toast';
import AddTorrentModal from './components/AddTorrentModal';
import { sendRpc } from './rpc';
import { Torrent } from './types';

export interface ToastMessage {
    id: string;
    type: 'success' | 'error' | 'info';
    message: string;
}

function App() {
    const [view, setView] = useState<'home' | 'library' | 'search'>('home');
    const [torrents, setTorrents] = useState<Torrent[]>([]);
    const [activeStreamUrl, setActiveStreamUrl] = useState<string | null>(null);
    const [showSettings, setShowSettings] = useState(false);
    const [showAddTorrent, setShowAddTorrent] = useState(false);
    const [selectedTorrent, setSelectedTorrent] = useState<Torrent | null>(null);
    const [toasts, setToasts] = useState<ToastMessage[]>([]);
    const [isDragging, setIsDragging] = useState(false);

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

    useEffect(() => {
        const fetchTorrents = async () => {
            try {
                const resp = await sendRpc('ListTorrents');
                if (resp && resp.result) {
                    setTorrents(resp.result);
                }
            } catch (e) {
                console.error('Failed to fetch torrents:', e);
            }
        };

        fetchTorrents();
        const interval = setInterval(fetchTorrents, 1000);
        return () => clearInterval(interval);
    }, []);

    const handleStreamStart = async (id: string) => {
        const resp = await sendRpc('StreamTorrent', { id });
        if (resp?.result?.url) {
            setActiveStreamUrl(resp.result.url);
            addToast('info', 'Starting stream...');
        } else {
            addToast('error', resp?.error || 'Failed to start stream');
        }
    };

    const handleAddTorrent = async (magnet: string) => {
        const resp = await sendRpc('AddTorrent', { magnet });
        if (resp?.result) {
            addToast('success', `Added: ${resp.result.name || 'New torrent'}`);
            setShowAddTorrent(false);
        } else {
            addToast('error', resp?.error || 'Failed to add torrent');
        }
    };

    const handleAddTorrentFile = async (file: File) => {
        const reader = new FileReader();
        reader.onload = async (e) => {
            const content = e.target?.result as string;
            const resp = await sendRpc('AddTorrentFile', { 
                name: file.name, 
                content: content.split(',')[1] || content 
            });
            if (resp?.result) {
                addToast('success', `Added: ${resp.result.name || file.name}`);
                setShowAddTorrent(false);
            } else {
                addToast('error', resp?.error || 'Failed to add torrent file');
            }
        };
        reader.readAsDataURL(file);
    };

    const handlePauseTorrent = async (id: string) => {
        const resp = await sendRpc('PauseTorrent', { id });
        if (resp?.result) {
            addToast('info', 'Torrent paused');
        }
    };

    const handleResumeTorrent = async (id: string) => {
        const resp = await sendRpc('StartTorrent', { id });
        if (resp?.result) {
            addToast('info', 'Torrent resumed');
        }
    };

    const handleRemoveTorrent = async (id: string) => {
        const resp = await sendRpc('RemoveTorrent', { id });
        if (resp?.result) {
            addToast('success', 'Torrent removed');
            setSelectedTorrent(null);
        } else {
            addToast('error', resp?.error || 'Failed to remove torrent');
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
        t.status === 'Downloading' || t.status === 'Streaming'
    ) || torrents[0];

    return (
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
                {activeStreamUrl && (
                    <VideoPlayer
                        streamUrl={activeStreamUrl}
                        onClose={() => setActiveStreamUrl(null)}
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
                />

                <main className="flex-1 overflow-y-auto p-6">
                    <header className="flex justify-between items-center mb-8">
                        <div>
                            <h1 className="text-3xl font-bold text-aurora-text">
                                {view === 'home' ? 'Welcome Back' : view === 'library' ? 'Your Library' : 'Search'}
                            </h1>
                            <p className="text-aurora-dim mt-1">
                                {torrents.length} torrents • {activeDownloads} active
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
                            {/* Quick stats */}
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Active Downloads</div>
                                    <div className="text-3xl font-bold text-aurora-cyan">{activeDownloads}</div>
                                </div>
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Download Speed</div>
                                    <div className="text-3xl font-bold text-aurora-teal">
                                        {formatSpeed(torrents.reduce((sum, t) => sum + t.download_speed, 0))}
                                    </div>
                                </div>
                                <div className="card p-6">
                                    <div className="text-aurora-dim text-sm mb-1">Upload Speed</div>
                                    <div className="text-3xl font-bold text-aurora-violet">
                                        {formatSpeed(torrents.reduce((sum, t) => sum + t.upload_speed, 0))}
                                    </div>
                                </div>
                            </div>

                            {/* Recent/Active torrents */}
                            {torrents.length > 0 ? (
                                <div>
                                    <h2 className="text-xl font-semibold mb-4">Recent Activity</h2>
                                    <LibraryGrid 
                                        torrents={torrents.slice(0, 5)} 
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
                            torrents={torrents} 
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
                                Search through your torrents and discover new content
                            </p>
                        </motion.div>
                    )}
                </main>
            </div>

            <NowPlayingFooter 
                torrent={activeTorrent} 
                onStream={activeTorrent ? () => handleStreamStart(activeTorrent.id) : undefined}
            />
        </div>
    );
}

function formatSpeed(bytes: number): string {
    if (bytes === 0) return '0 B/s';
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export default App;
