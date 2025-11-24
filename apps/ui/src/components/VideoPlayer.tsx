import { X } from 'lucide-react';

interface VideoPlayerProps {
    streamUrl: string;
    onClose: () => void;
}

export default function VideoPlayer({ streamUrl, onClose }: VideoPlayerProps) {
    return (
        <div className="fixed inset-0 bg-black/90 z-[100] flex items-center justify-center p-8">
            <button
                onClick={onClose}
                className="absolute top-4 right-4 text-white hover:text-spotify-green transition"
            >
                <X size={32} />
            </button>
            <div className="w-full max-w-5xl aspect-video bg-black rounded-lg overflow-hidden shadow-2xl border border-spotify-grey/20">
                <video
                    src={streamUrl}
                    controls
                    autoPlay
                    className="w-full h-full object-contain"
                >
                    Your browser does not support the video tag.
                </video>
            </div>
        </div>
    );
}
