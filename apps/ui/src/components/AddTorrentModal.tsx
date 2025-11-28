import { useState, useRef } from 'react';
import { motion } from 'framer-motion';
import { X, Link, Upload, Clipboard } from 'lucide-react';

interface AddTorrentModalProps {
    onClose: () => void;
    onAddMagnet: (magnet: string) => void;
    onAddFile: (file: File) => void;
}

export default function AddTorrentModal({ onClose, onAddMagnet, onAddFile }: AddTorrentModalProps) {
    const [magnetLink, setMagnetLink] = useState('');
    const [isDragging, setIsDragging] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (magnetLink.trim()) {
            onAddMagnet(magnetLink.trim());
        }
    };

    const handlePaste = async () => {
        try {
            const text = await navigator.clipboard.readText();
            if (text.startsWith('magnet:')) {
                setMagnetLink(text);
            }
        } catch (e) {
            console.error('Failed to read clipboard:', e);
        }
    };

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file && file.name.endsWith('.torrent')) {
            onAddFile(file);
        }
    };

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
        
        const file = Array.from(e.dataTransfer.files).find(f => f.name.endsWith('.torrent'));
        if (file) {
            onAddFile(file);
        }
    };

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
                    <h2 className="text-xl font-bold text-aurora-text">Add Torrent</h2>
                    <button
                        onClick={onClose}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded-lg transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                {/* Magnet link input */}
                <form onSubmit={handleSubmit} className="mb-6">
                    <label className="block text-sm font-medium text-aurora-dim mb-2">
                        Magnet Link
                    </label>
                    <div className="flex gap-2">
                        <div className="relative flex-1">
                            <Link className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-aurora-muted" />
                            <input
                                type="text"
                                value={magnetLink}
                                onChange={(e) => setMagnetLink(e.target.value)}
                                placeholder="magnet:?xt=urn:btih:..."
                                className="input pl-10"
                            />
                        </div>
                        <button
                            type="button"
                            onClick={handlePaste}
                            className="btn-secondary px-3"
                            title="Paste from clipboard"
                        >
                            <Clipboard size={18} />
                        </button>
                    </div>
                    <button
                        type="submit"
                        disabled={!magnetLink.trim()}
                        className="btn-primary w-full mt-3 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        Add Magnet
                    </button>
                </form>

                {/* Divider */}
                <div className="flex items-center gap-4 mb-6">
                    <div className="flex-1 h-px bg-aurora-border" />
                    <span className="text-sm text-aurora-muted">or</span>
                    <div className="flex-1 h-px bg-aurora-border" />
                </div>

                {/* File upload */}
                <div
                    onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
                    onDragLeave={(e) => { e.preventDefault(); setIsDragging(false); }}
                    onDrop={handleDrop}
                    onClick={() => fileInputRef.current?.click()}
                    className={`border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all ${
                        isDragging 
                            ? 'border-aurora-cyan bg-aurora-cyan/5' 
                            : 'border-aurora-border hover:border-aurora-cyan/50 hover:bg-aurora-night/30'
                    }`}
                >
                    <Upload className={`w-10 h-10 mx-auto mb-3 ${isDragging ? 'text-aurora-cyan' : 'text-aurora-muted'}`} />
                    <p className="text-sm text-aurora-text mb-1">
                        Drop a .torrent file here
                    </p>
                    <p className="text-xs text-aurora-muted">
                        or click to browse
                    </p>
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept=".torrent"
                        onChange={handleFileChange}
                        className="hidden"
                    />
                </div>
            </motion.div>
        </motion.div>
    );
}

