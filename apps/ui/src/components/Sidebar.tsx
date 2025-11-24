import { Home, Library, Search, PlusSquare, Download } from 'lucide-react';

export default function Sidebar({ currentView, setView }: { currentView: string, setView: (v: string) => void }) {
    return (
        <div className="w-64 bg-black flex flex-col gap-2 p-2">
            <div className="bg-spotify-dark rounded-lg p-4 flex flex-col gap-4">
                <button onClick={() => setView('home')} className={`flex items-center gap-4 font-bold transition ${currentView === 'home' ? 'text-white' : 'text-spotify-grey hover:text-white'}`}>
                    <Home size={24} />
                    Home
                </button>
                <button onClick={() => setView('search')} className={`flex items-center gap-4 font-bold transition ${currentView === 'search' ? 'text-white' : 'text-spotify-grey hover:text-white'}`}>
                    <Search size={24} />
                    Search Torrents
                </button>
            </div>

            <div className="bg-spotify-dark rounded-lg p-4 flex-1 flex flex-col gap-4">
                <div className="flex justify-between items-center text-spotify-grey hover:text-white transition cursor-pointer" onClick={() => setView('library')}>
                    <div className="flex items-center gap-2 font-bold">
                        <Library size={24} />
                        Your Library
                    </div>
                    <PlusSquare size={20} />
                </div>

                <div className="flex flex-col gap-2 mt-4 overflow-y-auto">
                    {/* Active Downloads */}
                    <div className="flex items-center gap-3 p-2 hover:bg-spotify-light rounded cursor-pointer">
                        <div className="w-12 h-12 bg-gradient-to-br from-green-400 to-emerald-600 rounded flex items-center justify-center">
                            <Download size={20} className="text-white" />
                        </div>
                        <div>
                            <p className="font-bold text-white">Active Downloads</p>
                            <p className="text-sm text-spotify-grey">0 downloading</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
