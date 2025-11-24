import { Play } from 'lucide-react';
import { sendRpc } from '../rpc';

export default function LibraryGrid({ torrents, onStream }: { torrents: any[], onStream?: (id: string) => void }) {
    const addTorrent = async () => {
        const magnet = prompt("Enter Magnet Link:");
        if (magnet) {
            try {
                const res = await sendRpc('AddTorrent', { magnet });
                if (res && res.result) {
                    alert("Torrent added successfully!");
                } else {
                    alert("Failed to add torrent.");
                }
            } catch (e) {
                alert("Error adding torrent: " + e);
            }
        }
    };

    const streamTorrent = async (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        if (onStream) {
            onStream(id);
        } else {
            await sendRpc('StreamTorrent', { id });
        }
    };

    return (
        <div>
            <div className="flex justify-between items-end mb-6">
                <h2 className="text-2xl font-bold">Your Torrents</h2>
                <button onClick={addTorrent} className="text-sm font-bold hover:underline text-spotify-grey hover:text-white">Show all</button>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                {/* Add New Card */}
                <div onClick={addTorrent} className="bg-spotify-dark p-4 rounded-lg hover:bg-spotify-light transition group cursor-pointer">
                    <div className="aspect-square bg-spotify-light rounded-md mb-4 flex items-center justify-center shadow-lg group-hover:shadow-xl">
                        <span className="text-4xl">+</span>
                    </div>
                    <h3 className="font-bold truncate mb-1">Add Torrent</h3>
                    <p className="text-sm text-spotify-grey line-clamp-2">Paste magnet link</p>
                </div>

                {torrents.map((t: any) => (
                    <div key={t.id} className="bg-spotify-dark p-4 rounded-lg hover:bg-spotify-light transition group cursor-pointer relative">
                        <div className="aspect-square bg-gradient-to-br from-green-400 to-blue-500 rounded-md mb-4 shadow-lg group-hover:shadow-xl relative">
                            <button onClick={(e) => streamTorrent(t.id, e)} className="absolute bottom-2 right-2 bg-spotify-green rounded-full p-3 shadow-lg translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition hover:scale-105">
                                <Play fill="black" className="text-black ml-1" size={20} />
                            </button>
                        </div>
                        <h3 className="font-bold truncate mb-1">{t.name}</h3>
                        <p className="text-sm text-spotify-grey line-clamp-2">{t.status} • {Math.round(t.progress * 100)}%</p>
                    </div>
                ))}
            </div>
        </div>
    );
}
