import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { X, Save, Folder, ArrowDown, ArrowUp } from 'lucide-react';
import { sendRpc } from '../rpc';

interface SettingsModalProps {
    onClose: () => void;
}

export default function SettingsModal({ onClose }: SettingsModalProps) {
    const [downloadPath, setDownloadPath] = useState('');
    const [maxDownloadSpeed, setMaxDownloadSpeed] = useState(0);
    const [maxUploadSpeed, setMaxUploadSpeed] = useState(0);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const resp = await sendRpc('GetConfig');
                if (resp?.result) {
                    setDownloadPath(resp.result.download_path || '');
                    setMaxDownloadSpeed(resp.result.max_download_speed || 0);
                    setMaxUploadSpeed(resp.result.max_upload_speed || 0);
                }
            } catch (error) {
                console.error('Failed to fetch settings:', error);
            } finally {
                setLoading(false);
            }
        };
        fetchConfig();
    }, []);

    const handleSave = async () => {
        setSaving(true);
        try {
            await sendRpc('SetConfig', {
                download_path: downloadPath,
                max_download_speed: maxDownloadSpeed,
                max_upload_speed: maxUploadSpeed,
            });
            onClose();
        } catch (error) {
            console.error('Failed to save settings:', error);
        } finally {
            setSaving(false);
        }
    };

    const formatSpeedLabel = (bytes: number) => {
        if (bytes === 0) return 'Unlimited';
        const mb = bytes / (1024 * 1024);
        return `${mb.toFixed(1)} MB/s`;
    };

    if (loading) {
        return (
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="fixed inset-0 bg-aurora-void/80 backdrop-blur-sm z-[100] flex items-center justify-center"
            >
                <div className="w-8 h-8 border-2 border-aurora-cyan border-t-transparent rounded-full animate-spin" />
            </motion.div>
        );
    }

    return (
        <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-aurora-void/80 backdrop-blur-sm z-[100] flex items-center justify-center p-4"
            onClick={onClose}
        >
            <motion.div
                initial={{ opacity: 0, scale: 0.95, y: 20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: 20 }}
                className="card w-full max-w-lg p-6"
                onClick={e => e.stopPropagation()}
            >
                <div className="flex items-center justify-between mb-6">
                    <h2 className="text-xl font-bold text-aurora-text">Settings</h2>
                    <button
                        onClick={onClose}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded-lg transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                <div className="space-y-6">
                    {/* Download Path */}
                    <div>
                        <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                            <Folder size={16} />
                            Download Location
                        </label>
                        <input
                            type="text"
                            value={downloadPath}
                            onChange={(e) => setDownloadPath(e.target.value)}
                            className="input"
                            placeholder="/path/to/downloads"
                        />
                    </div>

                    {/* Speed Limits */}
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                                <ArrowDown size={16} className="text-aurora-cyan" />
                                Max Download
                            </label>
                            <input
                                type="number"
                                min="0"
                                step="1048576"
                                value={maxDownloadSpeed}
                                onChange={(e) => setMaxDownloadSpeed(Math.max(0, Number(e.target.value)))}
                                className="input"
                            />
                            <p className="text-xs text-aurora-muted mt-1">
                                {formatSpeedLabel(maxDownloadSpeed)}
                            </p>
                        </div>
                        <div>
                            <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                                <ArrowUp size={16} className="text-aurora-teal" />
                                Max Upload
                            </label>
                            <input
                                type="number"
                                min="0"
                                step="1048576"
                                value={maxUploadSpeed}
                                onChange={(e) => setMaxUploadSpeed(Math.max(0, Number(e.target.value)))}
                                className="input"
                            />
                            <p className="text-xs text-aurora-muted mt-1">
                                {formatSpeedLabel(maxUploadSpeed)}
                            </p>
                        </div>
                    </div>

                    <p className="text-xs text-aurora-muted">
                        Set to 0 for unlimited speed. Values are in bytes per second.
                    </p>
                </div>

                <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-aurora-border/50">
                    <button onClick={onClose} className="btn-secondary">
                        Cancel
                    </button>
                    <button
                        onClick={handleSave}
                        disabled={saving}
                        className="btn-primary flex items-center gap-2"
                    >
                        {saving ? (
                            <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                        ) : (
                            <Save size={18} />
                        )}
                        Save Changes
                    </button>
                </div>
            </motion.div>
        </motion.div>
    );
}
