import { motion } from 'framer-motion';
import { 
    X, Maximize2, Volume2, VolumeX, ExternalLink, AlertCircle, 
    Loader2, RefreshCw, Settings, Play, Pause, SkipBack, SkipForward,
    Minimize2
} from 'lucide-react';
import { useState, useRef, useCallback, useEffect } from 'react';
import { open } from '@tauri-apps/api/shell';
import Hls from 'hls.js';

interface VideoPlayerProps {
    streamUrl: string;
    torrentId: string;
    fileIndex: number;
    fileName?: string;
    onClose: () => void;
}

type PlayerState = 'loading' | 'checking' | 'playing' | 'error' | 'unsupported' | 'transcoding';

interface MediaInfo {
    file_name: string;
    file_size: number;
    mime_type: string;
    extension: string;
    native_supported: boolean;
    transcode_available: boolean;
    recommend_transcode: boolean;
    streaming_url: string;
    direct_url: string;
    transcode_url: string | null;
}

export default function VideoPlayer({ streamUrl, torrentId, fileIndex, fileName, onClose }: VideoPlayerProps) {
    const [isMuted, setIsMuted] = useState(false);
    const [playerState, setPlayerState] = useState<PlayerState>('checking');
    const [errorMessage, setErrorMessage] = useState<string>('');
    const [isBuffering, setIsBuffering] = useState(false);
    const [progress, setProgress] = useState(0);
    const [duration, setDuration] = useState(0);
    const [currentTime, setCurrentTime] = useState(0);
    const [isFullscreen, setIsFullscreen] = useState(false);
    const [showControls, setShowControls] = useState(true);
    const [isPlaying, setIsPlaying] = useState(false);
    const [volume, setVolume] = useState(1);
    const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
    const [useTranscoding, setUseTranscoding] = useState(false);
    const [playbackRate, setPlaybackRate] = useState(1);
    
    const containerRef = useRef<HTMLDivElement>(null);
    const videoRef = useRef<HTMLVideoElement>(null);
    const hlsRef = useRef<Hls | null>(null);
    const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    // Fetch media info to determine best playback method
    useEffect(() => {
        const fetchMediaInfo = async () => {
            try {
                const response = await fetch(`http://127.0.0.1:3000/info/${torrentId}/${fileIndex}`);
                if (response.ok) {
                    const info: MediaInfo = await response.json();
                    setMediaInfo(info);
                    
                    // Auto-select transcoding if recommended
                    if (info.recommend_transcode) {
                        setUseTranscoding(true);
                    }
                    
                    setPlayerState('loading');
                } else {
                    setPlayerState('loading');
                }
            } catch (e) {
                console.error('Failed to fetch media info:', e);
                setPlayerState('loading');
            }
        };
        
        fetchMediaInfo();
    }, [torrentId, fileIndex]);

    // Setup HLS.js when using transcoding
    useEffect(() => {
        if (playerState !== 'loading' || !mediaInfo) return;
        
        const video = videoRef.current;
        if (!video) return;

        const sourceUrl = useTranscoding && mediaInfo.transcode_url 
            ? mediaInfo.transcode_url 
            : mediaInfo.direct_url;

        // Clean up previous HLS instance
        if (hlsRef.current) {
            hlsRef.current.destroy();
            hlsRef.current = null;
        }

        if (useTranscoding && Hls.isSupported() && mediaInfo.transcode_url) {
            // Use HLS.js for transcoded streams
            setPlayerState('transcoding');
            
            const hls = new Hls({
                enableWorker: true,
                lowLatencyMode: true,
                backBufferLength: 90,
            });
            
            hlsRef.current = hls;
            
            hls.loadSource(mediaInfo.transcode_url);
            hls.attachMedia(video);
            
            hls.on(Hls.Events.MANIFEST_PARSED, () => {
                video.play().catch(() => {});
            });
            
            hls.on(Hls.Events.ERROR, (_, data) => {
                if (data.fatal) {
                    switch (data.type) {
                        case Hls.ErrorTypes.NETWORK_ERROR:
                            setErrorMessage('Network error during transcoding');
                            hls.startLoad();
                            break;
                        case Hls.ErrorTypes.MEDIA_ERROR:
                            hls.recoverMediaError();
                            break;
                        default:
                            setPlayerState('error');
                            setErrorMessage('HLS playback error');
                            break;
                    }
                }
            });
        } else if (video.canPlayType('application/vnd.apple.mpegurl') && useTranscoding && mediaInfo.transcode_url) {
            // Native HLS support (Safari)
            video.src = mediaInfo.transcode_url;
            video.play().catch(() => {});
        } else {
            // Direct streaming
            video.src = sourceUrl;
            video.play().catch(() => {});
        }

        return () => {
            if (hlsRef.current) {
                hlsRef.current.destroy();
                hlsRef.current = null;
            }
        };
    }, [playerState, mediaInfo, useTranscoding]);

    const toggleFullscreen = useCallback(() => {
        if (!document.fullscreenElement) {
            containerRef.current?.requestFullscreen();
            setIsFullscreen(true);
        } else {
            document.exitFullscreen();
            setIsFullscreen(false);
        }
    }, []);

    const openInExternalPlayer = useCallback(async () => {
        try {
            const url = mediaInfo?.direct_url || streamUrl;
            await open(url);
        } catch (e) {
            console.error('Failed to open external player:', e);
            setErrorMessage('Failed to open external player');
        }
    }, [streamUrl, mediaInfo]);

    const handleCanPlay = useCallback(() => {
        setPlayerState('playing');
        setIsBuffering(false);
    }, []);

    const handleError = useCallback((e: React.SyntheticEvent<HTMLVideoElement>) => {
        const video = e.currentTarget;
        const error = video.error;
        
        // If using direct streaming and it fails, try transcoding
        if (!useTranscoding && mediaInfo?.transcode_available) {
            setUseTranscoding(true);
            setPlayerState('loading');
            return;
        }
        
        let message = 'Unknown error occurred';
        if (error) {
            switch (error.code) {
                case MediaError.MEDIA_ERR_ABORTED:
                    message = 'Playback was aborted';
                    break;
                case MediaError.MEDIA_ERR_NETWORK:
                    message = 'Network error - file may still be downloading';
                    setPlayerState('error');
                    setErrorMessage(message);
                    return;
                case MediaError.MEDIA_ERR_DECODE:
                    message = 'Codec not supported by browser';
                    if (mediaInfo?.transcode_available && !useTranscoding) {
                        setUseTranscoding(true);
                        setPlayerState('loading');
                        return;
                    }
                    setPlayerState('unsupported');
                    setErrorMessage(message);
                    return;
                case MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED:
                    message = 'Format not supported by browser';
                    if (mediaInfo?.transcode_available && !useTranscoding) {
                        setUseTranscoding(true);
                        setPlayerState('loading');
                        return;
                    }
                    setPlayerState('unsupported');
                    setErrorMessage(message);
                    return;
            }
        }
        
        setPlayerState('error');
        setErrorMessage(message);
    }, [mediaInfo, useTranscoding]);

    const handleWaiting = useCallback(() => {
        setIsBuffering(true);
    }, []);

    const handlePlaying = useCallback(() => {
        setIsBuffering(false);
        setPlayerState('playing');
        setIsPlaying(true);
    }, []);

    const handlePause = useCallback(() => {
        setIsPlaying(false);
    }, []);

    const handleTimeUpdate = useCallback(() => {
        const video = videoRef.current;
        if (video) {
            setCurrentTime(video.currentTime);
            setProgress((video.currentTime / video.duration) * 100 || 0);
        }
    }, []);

    const handleLoadedMetadata = useCallback(() => {
        const video = videoRef.current;
        if (video) {
            setDuration(video.duration);
        }
    }, []);

    const handleSeek = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
        const video = videoRef.current;
        if (video && duration) {
            const rect = e.currentTarget.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const percentage = x / rect.width;
            video.currentTime = percentage * duration;
        }
    }, [duration]);

    const handleVolumeChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const value = parseFloat(e.target.value);
        setVolume(value);
        if (videoRef.current) {
            videoRef.current.volume = value;
        }
        setIsMuted(value === 0);
    }, []);

    const toggleMute = useCallback(() => {
        if (videoRef.current) {
            videoRef.current.muted = !isMuted;
            setIsMuted(!isMuted);
        }
    }, [isMuted]);

    const togglePlay = useCallback(() => {
        const video = videoRef.current;
        if (video) {
            if (video.paused) {
                video.play();
            } else {
                video.pause();
            }
        }
    }, []);

    const skip = useCallback((seconds: number) => {
        const video = videoRef.current;
        if (video) {
            video.currentTime = Math.max(0, Math.min(video.currentTime + seconds, duration));
        }
    }, [duration]);

    const formatTime = (seconds: number): string => {
        if (!isFinite(seconds)) return '0:00';
        const hrs = Math.floor(seconds / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        const secs = Math.floor(seconds % 60);
        if (hrs > 0) {
            return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        }
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    const retry = useCallback(() => {
        setPlayerState('loading');
        setErrorMessage('');
        if (videoRef.current) {
            videoRef.current.load();
        }
    }, []);

    const switchPlaybackMethod = useCallback(() => {
        setUseTranscoding(!useTranscoding);
        setPlayerState('loading');
    }, [useTranscoding]);

    // Auto-hide controls
    const showControlsTemporarily = useCallback(() => {
        setShowControls(true);
        if (controlsTimeoutRef.current) {
            clearTimeout(controlsTimeoutRef.current);
        }
        controlsTimeoutRef.current = setTimeout(() => {
            if (isPlaying) {
                setShowControls(false);
            }
        }, 3000);
    }, [isPlaying]);

    // Handle keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            switch (e.key) {
                case 'Escape':
                    onClose();
                    break;
                case ' ':
                case 'k':
                    e.preventDefault();
                    togglePlay();
                    break;
                case 'f':
                    toggleFullscreen();
                    break;
                case 'm':
                    toggleMute();
                    break;
                case 'ArrowLeft':
                    skip(-10);
                    break;
                case 'ArrowRight':
                    skip(10);
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    setVolume(v => Math.min(1, v + 0.1));
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    setVolume(v => Math.max(0, v - 0.1));
                    break;
            }
            showControlsTemporarily();
        };
        
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onClose, togglePlay, toggleFullscreen, toggleMute, skip, showControlsTemporarily]);

    // Update volume when state changes
    useEffect(() => {
        if (videoRef.current) {
            videoRef.current.volume = volume;
        }
    }, [volume]);

    // Fullscreen change handler
    useEffect(() => {
        const handleFullscreenChange = () => {
            setIsFullscreen(!!document.fullscreenElement);
        };
        document.addEventListener('fullscreenchange', handleFullscreenChange);
        return () => document.removeEventListener('fullscreenchange', handleFullscreenChange);
    }, []);

    return (
        <motion.div
            ref={containerRef}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-aurora-void z-[100] flex flex-col"
            onMouseMove={showControlsTemporarily}
        >
            {/* Header */}
            <motion.div 
                initial={{ y: -50, opacity: 0 }}
                animate={{ y: showControls ? 0 : -50, opacity: showControls ? 1 : 0 }}
                className="absolute top-0 left-0 right-0 p-4 bg-gradient-to-b from-black/90 to-transparent z-20 flex items-center justify-between"
            >
                <div className="flex items-center gap-4">
                    <h3 className="text-lg font-medium text-aurora-text truncate max-w-md">
                        {fileName || mediaInfo?.file_name || 'Now Playing'}
                    </h3>
                    {useTranscoding && (
                        <span className="px-2 py-1 bg-aurora-violet/20 text-aurora-violet text-xs rounded-full">
                            Transcoding
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-2">
                    {mediaInfo?.transcode_available && (
                        <button
                            onClick={switchPlaybackMethod}
                            className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                            title={useTranscoding ? "Switch to direct play" : "Switch to transcoding"}
                        >
                            <Settings size={20} />
                        </button>
                    )}
                    <button
                        onClick={openInExternalPlayer}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                        title="Open in external player (VLC, mpv, etc.)"
                    >
                        <ExternalLink size={20} />
                    </button>
                    <button
                        onClick={toggleFullscreen}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                    >
                        {isFullscreen ? <Minimize2 size={20} /> : <Maximize2 size={20} />}
                    </button>
                    <button
                        onClick={onClose}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-white/10 rounded-lg transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>
            </motion.div>

            {/* Video Container */}
            <div 
                className="flex-1 flex items-center justify-center bg-black relative"
                onClick={togglePlay}
            >
                {/* Loading/Checking State */}
                {(playerState === 'loading' || playerState === 'checking') && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/80 z-20">
                        <Loader2 className="w-12 h-12 text-aurora-cyan animate-spin mb-4" />
                        <p className="text-aurora-dim">
                            {playerState === 'checking' ? 'Checking media format...' : 'Loading media...'}
                        </p>
                    </div>
                )}

                {/* Transcoding State */}
                {playerState === 'transcoding' && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/80 z-20">
                        <Loader2 className="w-12 h-12 text-aurora-violet animate-spin mb-4" />
                        <p className="text-aurora-text font-medium">Transcoding...</p>
                        <p className="text-aurora-dim text-sm mt-2">Converting to browser-compatible format</p>
                    </div>
                )}

                {/* Buffering Indicator */}
                {isBuffering && playerState === 'playing' && (
                    <div className="absolute inset-0 flex items-center justify-center bg-black/50 z-20 pointer-events-none">
                        <Loader2 className="w-12 h-12 text-aurora-cyan animate-spin" />
                    </div>
                )}

                {/* Error State */}
                {playerState === 'error' && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-black z-20 p-8">
                        <AlertCircle className="w-16 h-16 text-aurora-rose mb-4" />
                        <h4 className="text-xl font-semibold text-aurora-text mb-2">Playback Error</h4>
                        <p className="text-aurora-dim text-center mb-6 max-w-md">{errorMessage}</p>
                        <div className="flex gap-4">
                            <button onClick={retry} className="btn-secondary flex items-center gap-2">
                                <RefreshCw size={18} />
                                Retry
                            </button>
                            {mediaInfo?.transcode_available && !useTranscoding && (
                                <button onClick={switchPlaybackMethod} className="btn-secondary flex items-center gap-2">
                                    <Settings size={18} />
                                    Try Transcoding
                                </button>
                            )}
                            <button onClick={openInExternalPlayer} className="btn-primary flex items-center gap-2">
                                <ExternalLink size={18} />
                                Open in VLC
                            </button>
                        </div>
                    </div>
                )}

                {/* Unsupported Codec State */}
                {playerState === 'unsupported' && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-black z-20 p-8">
                        <AlertCircle className="w-16 h-16 text-aurora-violet mb-4" />
                        <h4 className="text-xl font-semibold text-aurora-text mb-2">Unsupported Format</h4>
                        <p className="text-aurora-dim text-center mb-2 max-w-md">
                            This video format cannot be played in the browser.
                        </p>
                        {!mediaInfo?.transcode_available && (
                            <p className="text-aurora-muted text-center mb-6 max-w-md text-sm">
                                Install FFmpeg to enable automatic transcoding, or use an external player.
                            </p>
                        )}
                        <div className="flex gap-4">
                            {mediaInfo?.transcode_available && (
                                <button onClick={switchPlaybackMethod} className="btn-secondary flex items-center gap-2">
                                    <Settings size={18} />
                                    Enable Transcoding
                                </button>
                            )}
                            <button onClick={openInExternalPlayer} className="btn-primary flex items-center gap-2">
                                <ExternalLink size={18} />
                                Open in VLC / mpv
                            </button>
                        </div>
                        <p className="text-aurora-muted text-xs mt-4">
                            Supported formats: MP4, WebM, MP3, OGG, WAV
                        </p>
                    </div>
                )}

                {/* Video Element */}
                <video
                    ref={videoRef}
                    controls={false}
                    autoPlay
                    muted={isMuted}
                    className={`max-w-full max-h-full ${playerState !== 'playing' ? 'opacity-0' : ''}`}
                    style={{ aspectRatio: '16/9' }}
                    onCanPlay={handleCanPlay}
                    onError={handleError}
                    onWaiting={handleWaiting}
                    onPlaying={handlePlaying}
                    onPause={handlePause}
                    onTimeUpdate={handleTimeUpdate}
                    onLoadedMetadata={handleLoadedMetadata}
                    playsInline
                >
                    Your browser does not support the video tag.
                </video>

                {/* Play/Pause overlay indicator */}
                {playerState === 'playing' && !isPlaying && (
                    <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <div className="w-20 h-20 rounded-full bg-black/50 flex items-center justify-center">
                            <Play size={40} className="text-white ml-1" />
                        </div>
                    </div>
                )}
            </div>

            {/* Custom Controls */}
            <motion.div 
                initial={{ y: 100, opacity: 0 }}
                animate={{ y: showControls ? 0 : 100, opacity: showControls ? 1 : 0 }}
                className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent z-20 px-4 pb-4 pt-16"
                onClick={e => e.stopPropagation()}
            >
                {/* Progress Bar */}
                <div 
                    className="h-1.5 bg-white/20 rounded-full cursor-pointer group mb-4"
                    onClick={handleSeek}
                >
                    <div 
                        className="h-full bg-aurora-cyan rounded-full relative transition-all group-hover:h-2"
                        style={{ width: `${progress}%` }}
                    >
                        <div className="absolute right-0 top-1/2 -translate-y-1/2 w-4 h-4 bg-aurora-cyan rounded-full opacity-0 group-hover:opacity-100 transition-opacity shadow-lg" />
                    </div>
                </div>

                {/* Control buttons */}
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        {/* Play/Pause */}
                        <button 
                            onClick={togglePlay}
                            className="p-2 text-white hover:text-aurora-cyan transition-colors"
                        >
                            {isPlaying ? <Pause size={24} /> : <Play size={24} />}
                        </button>

                        {/* Skip buttons */}
                        <button 
                            onClick={() => skip(-10)}
                            className="p-2 text-white/70 hover:text-white transition-colors"
                            title="Rewind 10s"
                        >
                            <SkipBack size={20} />
                        </button>
                        <button 
                            onClick={() => skip(10)}
                            className="p-2 text-white/70 hover:text-white transition-colors"
                            title="Forward 10s"
                        >
                            <SkipForward size={20} />
                        </button>

                        {/* Volume */}
                        <div className="flex items-center gap-2">
                            <button 
                                onClick={toggleMute}
                                className="p-2 text-white/70 hover:text-white transition-colors"
                            >
                                {isMuted || volume === 0 ? <VolumeX size={20} /> : <Volume2 size={20} />}
                            </button>
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.1"
                                value={isMuted ? 0 : volume}
                                onChange={handleVolumeChange}
                                className="w-20 h-1 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white"
                            />
                        </div>

                        {/* Time */}
                        <span className="text-sm text-white/70">
                            {formatTime(currentTime)} / {formatTime(duration)}
                        </span>
                    </div>

                    <div className="flex items-center gap-2">
                        {/* Playback speed */}
                        <select
                            value={playbackRate}
                            onChange={(e) => {
                                const rate = parseFloat(e.target.value);
                                setPlaybackRate(rate);
                                if (videoRef.current) videoRef.current.playbackRate = rate;
                            }}
                            className="bg-white/10 text-white text-sm rounded px-2 py-1 border-none outline-none cursor-pointer"
                        >
                            <option value="0.5">0.5x</option>
                            <option value="0.75">0.75x</option>
                            <option value="1">1x</option>
                            <option value="1.25">1.25x</option>
                            <option value="1.5">1.5x</option>
                            <option value="2">2x</option>
                        </select>

                        {/* External player */}
                        <button
                            onClick={openInExternalPlayer}
                            className="p-2 text-white/70 hover:text-white transition-colors"
                            title="Open in VLC"
                        >
                            <ExternalLink size={20} />
                        </button>

                        {/* Fullscreen */}
                        <button
                            onClick={toggleFullscreen}
                            className="p-2 text-white/70 hover:text-white transition-colors"
                        >
                            {isFullscreen ? <Minimize2 size={20} /> : <Maximize2 size={20} />}
                        </button>
                    </div>
                </div>
            </motion.div>

            {/* Keyboard shortcuts hint */}
            {showControls && playerState === 'playing' && (
                <div className="absolute bottom-20 left-1/2 -translate-x-1/2 px-4 py-2 bg-black/40 backdrop-blur-sm rounded-lg text-xs text-white/50 flex gap-4">
                    <span>Space: Play/Pause</span>
                    <span>←/→: Seek</span>
                    <span>F: Fullscreen</span>
                    <span>M: Mute</span>
                    <span>ESC: Close</span>
                </div>
            )}
        </motion.div>
    );
}
