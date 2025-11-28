import { motion } from 'framer-motion';
import { X, Maximize2, Volume2, VolumeX } from 'lucide-react';
import { useState, useRef } from 'react';

interface VideoPlayerProps {
    streamUrl: string;
    onClose: () => void;
}

export default function VideoPlayer({ streamUrl, onClose }: VideoPlayerProps) {
    const [isMuted, setIsMuted] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);

    const toggleFullscreen = () => {
        if (!document.fullscreenElement) {
            containerRef.current?.requestFullscreen();
        } else {
            document.exitFullscreen();
        }
    };

    return (
        <motion.div
            ref={containerRef}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-aurora-void z-[100] flex flex-col"
        >
            {/* Header */}
            <div className="absolute top-0 left-0 right-0 p-4 bg-gradient-to-b from-black/80 to-transparent z-10 flex items-center justify-between">
                <h3 className="text-lg font-medium text-aurora-text">Now Playing</h3>
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => setIsMuted(!isMuted)}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                    >
                        {isMuted ? <VolumeX size={20} /> : <Volume2 size={20} />}
                    </button>
                    <button
                        onClick={toggleFullscreen}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                    >
                        <Maximize2 size={20} />
                    </button>
                    <button
                        onClick={onClose}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>
            </div>

            {/* Video */}
            <div className="flex-1 flex items-center justify-center bg-black">
                <video
                    src={streamUrl}
                    controls
                    autoPlay
                    muted={isMuted}
                    className="max-w-full max-h-full"
                    style={{ aspectRatio: '16/9' }}
                >
                    Your browser does not support the video tag.
                </video>
            </div>

            {/* Buffering indicator overlay */}
            <div className="absolute bottom-4 left-1/2 -translate-x-1/2 px-4 py-2 bg-black/60 backdrop-blur-sm rounded-lg text-sm text-aurora-dim">
                Streaming... • Press ESC to close
            </div>
        </motion.div>
    );
}
