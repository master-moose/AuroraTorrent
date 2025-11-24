import { useState, useEffect } from 'react';
import { X, Save } from 'lucide-react';
import { sendRpc } from '../rpc';

interface SettingsModalProps {
    onClose: () => void;
}

export default function SettingsModal({ onClose }: SettingsModalProps) {
    const [downloadPath, setDownloadPath] = useState('');
    const [maxDownloadSpeed, setMaxDownloadSpeed] = useState(0);
    const [maxUploadSpeed, setMaxUploadSpeed] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchConfig = async () => {
            const resp = await sendRpc('GetConfig');
            if (resp && resp.result) {
                setDownloadPath(resp.result.download_path);
                setMaxDownloadSpeed(resp.result.max_download_speed);
                setMaxUploadSpeed(resp.result.max_upload_speed);
            }
            setLoading(false);
        };
        fetchConfig();
    }, []);

    const handleSave = async () => {
        await sendRpc('SetConfig', {
            download_path: downloadPath,
            max_download_speed: maxDownloadSpeed,
            max_upload_speed: maxUploadSpeed,
        });
        onClose();
    };

    if (loading) return null;

    return (
        <div className="fixed inset-0 bg-black/80 z-[100] flex items-center justify-center p-4">
            <div className="bg-spotify-dark w-full max-w-md rounded-lg p-6 relative shadow-2xl border border-spotify-light">
                <button onClick={onClose} className="absolute top-4 right-4 text-spotify-grey hover:text-white">
                    <X size={24} />
                </button>

                <h2 className="text-2xl font-bold mb-6">Settings</h2>

                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-bold mb-2 text-spotify-grey">Download Path</label>
                        <input
                            type="text"
                            value={downloadPath}
                            onChange={(e) => setDownloadPath(e.target.value)}
                            className="w-full bg-black border border-spotify-light rounded p-2 text-white focus:border-spotify-green focus:outline-none"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-bold mb-2 text-spotify-grey">Max Download Speed (bytes/s, 0 = unlimited)</label>
                        <input
                            type="number"
                            value={maxDownloadSpeed}
                            onChange={(e) => setMaxDownloadSpeed(Number(e.target.value))}
                            className="w-full bg-black border border-spotify-light rounded p-2 text-white focus:border-spotify-green focus:outline-none"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-bold mb-2 text-spotify-grey">Max Upload Speed (bytes/s, 0 = unlimited)</label>
                        <input
                            type="number"
                            value={maxUploadSpeed}
                            onChange={(e) => setMaxUploadSpeed(Number(e.target.value))}
                            className="w-full bg-black border border-spotify-light rounded p-2 text-white focus:border-spotify-green focus:outline-none"
                        />
                    </div>
                </div>

                <div className="mt-8 flex justify-end">
                    <button
                        onClick={handleSave}
                        className="bg-spotify-green text-black font-bold py-2 px-6 rounded-full hover:scale-105 transition flex items-center gap-2"
                    >
                        <Save size={18} />
                        Save
                    </button>
                </div>
            </div>
        </div>
    );
}
