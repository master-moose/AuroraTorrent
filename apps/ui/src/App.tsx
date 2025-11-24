import { useState, useEffect } from 'react';
import Sidebar from './components/Sidebar';
import LibraryGrid from './components/LibraryGrid';
import NowPlayingFooter from './components/NowPlayingFooter';
import VideoPlayer from './components/VideoPlayer';
import SettingsModal from './components/SettingsModal';
import TorrentDetails from './components/TorrentDetails';
import { sendRpc } from './rpc';

function App() {
    const [view, setView] = useState('library');
    const [torrents, setTorrents] = useState([]);
    const [speedUnit, setSpeedUnit] = useState<'MB/s' | 'kB/s'>('MB/s');
    const [activeStreamUrl, setActiveStreamUrl] = useState<string | null>(null);
    const [showSettings, setShowSettings] = useState(false);
    const [selectedTorrent, setSelectedTorrent] = useState<any | null>(null);

    useEffect(() => {
        const interval = setInterval(async () => {
            const resp = await sendRpc('ListTorrents');
            if (resp && resp.result) {
                setTorrents(resp.result);

                // Check if any torrent is streaming and update activeStreamUrl if needed
                // For now, we rely on the user clicking "Play" to set the URL, 
                // but we could also sync it here if the backend sends the URL in the state.
            }
        }, 1000);
        return () => clearInterval(interval);
    }, []);

    const handleStreamStart = async (id: string) => {
        const resp = await sendRpc('StreamTorrent', { id });
        if (resp && resp.result && resp.result.url) {
            setActiveStreamUrl(resp.result.url);
        }
    };

    return (
        <div className="flex flex-col h-screen w-screen bg-black text-white font-sans relative">
            {activeStreamUrl && (
                <VideoPlayer
                    streamUrl={activeStreamUrl}
                    onClose={() => {
                        setActiveStreamUrl(null);
                        // Optionally send a "StopStream" RPC here if needed
                    }}
                />
            )}
            {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
            {selectedTorrent && <TorrentDetails torrent={selectedTorrent} onClose={() => setSelectedTorrent(null)} />}
            <div className="flex flex-1 overflow-hidden">
                <Sidebar currentView={view} setView={setView} />
                <main className="flex-1 bg-spotify-black overflow-y-auto rounded-lg m-2 ml-0 p-6">
                    <header className="flex justify-between items-center mb-6">
                        <div className="flex gap-4">
                            <button className="bg-black/40 rounded-full p-2 px-3 hover:bg-black/60 transition"><span>&lt;</span></button>
                            <button className="bg-black/40 rounded-full p-2 px-3 hover:bg-black/60 transition"><span>&gt;</span></button>
                        </div>
                        <div className="flex gap-4">
                            <button onClick={() => setShowSettings(true)} className="text-sm font-bold text-spotify-grey hover:text-white transition">Settings</button>
                        </div>
                    </header>

                    {view === 'library' && <LibraryGrid torrents={torrents} onStream={handleStreamStart} />}
                </main>
                <div className="w-72 bg-black p-4 hidden lg:block">
                    <h2 className="font-bold mb-4">Torrent Activity</h2>
                    <p className="text-spotify-grey text-sm">Global download/upload stats will appear here.</p>
                </div>
            </div>
            <NowPlayingFooter torrents={torrents} speedUnit={speedUnit} setSpeedUnit={setSpeedUnit} />
        </div>
    );
}

export default App;
