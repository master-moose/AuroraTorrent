import { useState } from 'react';
import { X, File, Users, Server } from 'lucide-react';

interface TorrentDetailsProps {
    torrent: any;
    onClose: () => void;
}

export default function TorrentDetails({ torrent, onClose }: TorrentDetailsProps) {
    const [activeTab, setActiveTab] = useState<'files' | 'peers' | 'trackers'>('files');

    return (
        <div className="fixed inset-0 bg-black/80 z-[90] flex items-center justify-center p-4">
            <div className="bg-spotify-dark w-full max-w-4xl h-[80vh] rounded-lg p-6 relative shadow-2xl border border-spotify-light flex flex-col">
                <button onClick={onClose} className="absolute top-4 right-4 text-spotify-grey hover:text-white">
                    <X size={24} />
                </button>

                <h2 className="text-2xl font-bold mb-2">{torrent.name}</h2>
                <div className="text-sm text-spotify-grey mb-6">
                    {(torrent.total_size / 1024 / 1024).toFixed(1)} MB • {torrent.status}
                </div>

                <div className="flex gap-6 border-b border-spotify-light mb-4">
                    <button
                        onClick={() => setActiveTab('files')}
                        className={`pb-2 text-sm font-bold flex items-center gap-2 ${activeTab === 'files' ? 'text-spotify-green border-b-2 border-spotify-green' : 'text-spotify-grey hover:text-white'}`}
                    >
                        <File size={16} /> Files
                    </button>
                    <button
                        onClick={() => setActiveTab('peers')}
                        className={`pb-2 text-sm font-bold flex items-center gap-2 ${activeTab === 'peers' ? 'text-spotify-green border-b-2 border-spotify-green' : 'text-spotify-grey hover:text-white'}`}
                    >
                        <Users size={16} /> Peers
                    </button>
                    <button
                        onClick={() => setActiveTab('trackers')}
                        className={`pb-2 text-sm font-bold flex items-center gap-2 ${activeTab === 'trackers' ? 'text-spotify-green border-b-2 border-spotify-green' : 'text-spotify-grey hover:text-white'}`}
                    >
                        <Server size={16} /> Trackers
                    </button>
                </div>

                <div className="flex-1 overflow-y-auto">
                    {activeTab === 'files' && (
                        <table className="w-full text-left text-sm text-spotify-grey">
                            <thead>
                                <tr className="border-b border-spotify-light/30">
                                    <th className="py-2">Name</th>
                                    <th className="py-2 text-right">Size</th>
                                    <th className="py-2 text-right">Progress</th>
                                </tr>
                            </thead>
                            <tbody>
                                {torrent.files?.map((file: any, i: number) => (
                                    <tr key={i} className="hover:bg-white/5">
                                        <td className="py-2 text-white">{file.name}</td>
                                        <td className="py-2 text-right">{(file.size / 1024 / 1024).toFixed(1)} MB</td>
                                        <td className="py-2 text-right">{(file.progress * 100).toFixed(0)}%</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}

                    {activeTab === 'peers' && (
                        <table className="w-full text-left text-sm text-spotify-grey">
                            <thead>
                                <tr className="border-b border-spotify-light/30">
                                    <th className="py-2">IP</th>
                                    <th className="py-2">Client</th>
                                    <th className="py-2 text-right">Down Speed</th>
                                    <th className="py-2 text-right">Up Speed</th>
                                </tr>
                            </thead>
                            <tbody>
                                {torrent.peers?.map((peer: any, i: number) => (
                                    <tr key={i} className="hover:bg-white/5">
                                        <td className="py-2 text-white">{peer.ip}</td>
                                        <td className="py-2">{peer.client}</td>
                                        <td className="py-2 text-right">{(peer.down_speed / 1024).toFixed(1)} kB/s</td>
                                        <td className="py-2 text-right">{(peer.up_speed / 1024).toFixed(1)} kB/s</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}

                    {activeTab === 'trackers' && (
                        <table className="w-full text-left text-sm text-spotify-grey">
                            <thead>
                                <tr className="border-b border-spotify-light/30">
                                    <th className="py-2">URL</th>
                                    <th className="py-2 text-right">Status</th>
                                </tr>
                            </thead>
                            <tbody>
                                {torrent.trackers?.map((tracker: any, i: number) => (
                                    <tr key={i} className="hover:bg-white/5">
                                        <td className="py-2 text-white truncate max-w-md">{tracker.url}</td>
                                        <td className="py-2 text-right text-spotify-green">{tracker.status}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            </div>
        </div>
    );
}
