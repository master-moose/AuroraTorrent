import { useMemo, useRef, useEffect, useState } from 'react';

interface PieceBarProps {
    /**
     * Array of piece states:
     * 0 = missing/not started
     * 1 = downloading/partial
     * 2 = complete/have
     */
    pieces: number[];
    /** Height of the bar in pixels */
    height?: number;
    /** Show tooltip on hover */
    showTooltip?: boolean;
    /** Class name for the container */
    className?: string;
}

/**
 * Visualizes which pieces of a torrent have been downloaded
 * Similar to qBittorrent's downloaded pieces bar
 */
export default function PieceBar({ 
    pieces, 
    height = 20,
    showTooltip = true,
    className = ''
}: PieceBarProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState({ width: 0 });
    const [hoverIndex, setHoverIndex] = useState<number | null>(null);
    const [hoverPos, setHoverPos] = useState({ x: 0, y: 0 });

    // Calculate statistics
    const stats = useMemo(() => {
        if (pieces.length === 0) {
            return { complete: 0, downloading: 0, missing: 0, total: 0, progress: 0 };
        }
        
        let complete = 0, downloading = 0, missing = 0;
        for (const p of pieces) {
            if (p === 2) complete++;
            else if (p === 1) downloading++;
            else missing++;
        }
        
        return {
            complete,
            downloading,
            missing,
            total: pieces.length,
            progress: (complete / pieces.length) * 100
        };
    }, [pieces]);

    // Handle resize
    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                setDimensions({ width: rect.width });
            }
        };

        updateDimensions();
        
        const resizeObserver = new ResizeObserver(updateDimensions);
        if (containerRef.current) {
            resizeObserver.observe(containerRef.current);
        }
        
        return () => resizeObserver.disconnect();
    }, []);

    // Draw the piece bar
    useEffect(() => {
        const canvas = canvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (!canvas || !ctx || dimensions.width === 0 || pieces.length === 0) return;

        const { width } = dimensions;
        const dpr = window.devicePixelRatio || 1;
        
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;
        ctx.scale(dpr, dpr);

        // Clear canvas
        ctx.clearRect(0, 0, width, height);

        // Background
        ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
        ctx.fillRect(0, 0, width, height);

        // Colors
        const colors = {
            missing: 'rgba(255, 255, 255, 0.05)',
            downloading: '#f59e0b', // amber
            complete: '#22d3ee',    // aurora-cyan
        };

        // If we have more pieces than pixels, we need to aggregate
        const pixelsPerPiece = width / pieces.length;

        if (pixelsPerPiece >= 1) {
            // Each piece gets at least 1 pixel
            for (let i = 0; i < pieces.length; i++) {
                const x = i * pixelsPerPiece;
                const pieceWidth = Math.max(pixelsPerPiece - 0.5, 1);
                
                ctx.fillStyle = pieces[i] === 2 ? colors.complete : 
                               pieces[i] === 1 ? colors.downloading : 
                               colors.missing;
                ctx.fillRect(x, 0, pieceWidth, height);
            }
        } else {
            // Multiple pieces per pixel - aggregate
            const piecesPerPixel = pieces.length / width;
            
            for (let px = 0; px < width; px++) {
                const startPiece = Math.floor(px * piecesPerPixel);
                const endPiece = Math.min(Math.floor((px + 1) * piecesPerPixel), pieces.length);
                
                // Count states in this pixel's range
                let complete = 0, downloading = 0;
                for (let i = startPiece; i < endPiece; i++) {
                    if (pieces[i] === 2) complete++;
                    else if (pieces[i] === 1) downloading++;
                }
                
                const total = endPiece - startPiece;
                const completeRatio = complete / total;
                const downloadingRatio = downloading / total;

                // Draw based on predominant state
                if (completeRatio > 0.5) {
                    ctx.fillStyle = colors.complete;
                } else if (downloadingRatio > 0.5) {
                    ctx.fillStyle = colors.downloading;
                } else if (completeRatio > 0 || downloadingRatio > 0) {
                    // Mixed state - use gradient based on progress
                    const progress = completeRatio + downloadingRatio * 0.5;
                    ctx.fillStyle = `rgba(34, 211, 238, ${progress})`;
                } else {
                    ctx.fillStyle = colors.missing;
                }
                
                ctx.fillRect(px, 0, 1, height);
            }
        }

        // Draw a subtle border
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        ctx.lineWidth = 1;
        ctx.strokeRect(0.5, 0.5, width - 1, height - 1);

    }, [pieces, dimensions, height]);

    // Handle mouse move for tooltip
    const handleMouseMove = (e: React.MouseEvent) => {
        if (!showTooltip || pieces.length === 0 || !containerRef.current) return;
        
        const rect = containerRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const pieceIndex = Math.floor((x / rect.width) * pieces.length);
        
        if (pieceIndex >= 0 && pieceIndex < pieces.length) {
            setHoverIndex(pieceIndex);
            setHoverPos({ x: e.clientX, y: e.clientY });
        }
    };

    const handleMouseLeave = () => {
        setHoverIndex(null);
    };

    const getPieceStatus = (state: number) => {
        switch (state) {
            case 0: return 'Missing';
            case 1: return 'Downloading';
            case 2: return 'Complete';
            default: return 'Unknown';
        }
    };

    return (
        <div className={`relative ${className}`}>
            <div 
                ref={containerRef} 
                className="w-full rounded overflow-hidden cursor-crosshair"
                style={{ height }}
                onMouseMove={handleMouseMove}
                onMouseLeave={handleMouseLeave}
            >
                <canvas ref={canvasRef} className="w-full h-full" />
            </div>
            
            {/* Statistics */}
            <div className="flex items-center justify-between mt-2 text-xs text-aurora-dim">
                <div className="flex items-center gap-4">
                    <div className="flex items-center gap-1">
                        <div className="w-2 h-2 rounded-sm bg-aurora-cyan" />
                        <span>{stats.complete} complete</span>
                    </div>
                    {stats.downloading > 0 && (
                        <div className="flex items-center gap-1">
                            <div className="w-2 h-2 rounded-sm bg-amber-500" />
                            <span>{stats.downloading} downloading</span>
                        </div>
                    )}
                    <div className="flex items-center gap-1">
                        <div className="w-2 h-2 rounded-sm bg-aurora-night" />
                        <span>{stats.missing} missing</span>
                    </div>
                </div>
                <span>{stats.total} pieces total</span>
            </div>

            {/* Tooltip */}
            {showTooltip && hoverIndex !== null && (
                <div
                    className="fixed z-50 px-3 py-2 text-xs bg-aurora-void border border-aurora-border rounded shadow-lg pointer-events-none"
                    style={{
                        left: hoverPos.x + 10,
                        top: hoverPos.y + 10,
                    }}
                >
                    <div className="text-aurora-text font-medium">
                        Piece #{hoverIndex}
                    </div>
                    <div className={`${
                        pieces[hoverIndex] === 2 ? 'text-aurora-cyan' :
                        pieces[hoverIndex] === 1 ? 'text-amber-500' :
                        'text-aurora-dim'
                    }`}>
                        {getPieceStatus(pieces[hoverIndex])}
                    </div>
                </div>
            )}
        </div>
    );
}

/**
 * Compact version for list views
 */
export function PieceBarCompact({ 
    pieces, 
    progress,
    className = '' 
}: { 
    pieces: number[];
    progress: number;
    className?: string;
}) {
    const hasData = pieces.length > 0;
    
    // If no piece data, show a simple progress bar
    if (!hasData) {
        return (
            <div className={`h-1.5 rounded-full bg-aurora-night/50 overflow-hidden ${className}`}>
                <div 
                    className="h-full bg-aurora-cyan transition-all duration-300"
                    style={{ width: `${progress * 100}%` }}
                />
            </div>
        );
    }

    // Calculate segments
    const segments = useMemo(() => {
        const numSegments = 50; // Fixed number of visual segments
        const piecesPerSegment = pieces.length / numSegments;
        const result: ('complete' | 'partial' | 'missing')[] = [];
        
        for (let i = 0; i < numSegments; i++) {
            const start = Math.floor(i * piecesPerSegment);
            const end = Math.min(Math.floor((i + 1) * piecesPerSegment), pieces.length);
            
            let complete = 0, downloading = 0;
            for (let j = start; j < end; j++) {
                if (pieces[j] === 2) complete++;
                else if (pieces[j] === 1) downloading++;
            }
            
            const total = end - start;
            if (complete === total) {
                result.push('complete');
            } else if (complete > 0 || downloading > 0) {
                result.push('partial');
            } else {
                result.push('missing');
            }
        }
        
        return result;
    }, [pieces]);

    return (
        <div className={`h-1.5 rounded-full bg-aurora-night/50 overflow-hidden flex ${className}`}>
            {segments.map((state, i) => (
                <div
                    key={i}
                    className={`flex-1 ${
                        state === 'complete' ? 'bg-aurora-cyan' :
                        state === 'partial' ? 'bg-amber-500/70' :
                        'bg-transparent'
                    }`}
                />
            ))}
        </div>
    );
}

