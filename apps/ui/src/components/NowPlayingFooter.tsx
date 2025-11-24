import { Play, Pause, SkipBack, SkipForward, Settings, ListMusic, MonitorSpeaker, Volume2 } from 'lucide-react';

export default function NowPlayingFooter({ torrents, speedUnit, setSpeedUnit }: { torrents: any[], speedUnit: 'MB/s' | 'kB/s', setSpeedUnit: (u: 'MB/s' | 'kB/s') => void }) {
    const active = torrents.find(t => t.status === 'Downloading' || t.status === 'Seeding' || t.status === 'Streaming') || torrents[0];

    const formatSpeed = (bytes: number) => {
        if (speedUnit === 'MB/s') return `${(bytes / 1024 / 1024).toFixed(2)} MB/s`;
        return `${(bytes / 1024).toFixed(0)} kB/s`;
    };

    const toggleUnit = () => {
        setSpeedUnit(speedUnit === 'MB/s' ? 'kB/s' : 'MB/s');
    };

    return (
        <div className="h-24 bg-spotify-dark border-t border-spotify-light px-4 flex justify-between items-center z-50">
            <div className="flex items-center gap-4 w-[30%]">
                {active ? (
                    <>
                        <div className="w-14 h-14 bg-gradient-to-br from-green-400 to-blue-500 rounded"></div>
                        <div>
                            <div className="font-sm hover:underline cursor-pointer">{active.name}</div>
                            <div className="text-xs text-spotify-grey">{active.status}</div>
                        </div>
                    </>
                ) : (
                    <div className="text-xs text-spotify-grey">No active torrents</div>
                )}
            </div>

            <div className="flex flex-col items-center max-w-[40%] w-full gap-2">
                {active?.status === 'Streaming' ? (
                    // Player Controls
                    <div className="flex items-center gap-6">
                        <button className="text-spotify-grey hover:text-white"><SkipBack size={20} fill="currentColor" /></button>
                        <button className="bg-white rounded-full p-2 hover:scale-105 transition">
                            <Pause size={20} fill="black" className="text-black" />
                        </button>
                        <button className="text-spotify-grey hover:text-white"><SkipForward size={20} fill="currentColor" /></button>
                    </div>
                ) : (
                    // Progress Bar & Speed
                    <div className="flex flex-col items-center w-full">
                        <div className="text-xs text-spotify-grey font-mono mb-1 cursor-pointer hover:text-white" onClick={toggleUnit}>
                            {active ? formatSpeed(active.download_speed) : '0.00 MB/s'}
                        </div>
                    </div>
                )}

                {active && (
                    <div className="w-full flex items-center gap-2 text-xs text-spotify-grey">
                        <span>{((active.progress || 0) * 100).toFixed(1)}%</span>
                        <div className="h-1 flex-1 bg-spotify-light rounded-full overflow-hidden group cursor-pointer">
                            <div className="h-full bg-spotify-green transition-all" style={{ width: `${(active.progress || 0) * 100}%` }}></div>
                        </div>
                        <span>{(active.total_size / 1024 / 1024).toFixed(0)} MB</span>
                    </div>
                )}
            </div>

            <div className="flex items-center justify-end gap-3 w-[30%] text-spotify-grey">
                {active?.status === 'Streaming' && (
                    <>
                        <Settings size={20} className="hover:text-white cursor-pointer" />
                        <ListMusic size={20} className="hover:text-white cursor-pointer" />
                        <MonitorSpeaker size={20} className="hover:text-white cursor-pointer" />
                        <div className="flex items-center gap-2 w-32">
                            <Volume2 size={20} className="hover:text-white cursor-pointer" />
                            <div className="h-1 flex-1 bg-spotify-light rounded-full overflow-hidden group cursor-pointer">
                                <div className="h-full w-2/3 bg-white group-hover:bg-spotify-green"></div>
                            </div>
                        </div>
                    </>
                )}
            </div>
        </div>
    );
}
