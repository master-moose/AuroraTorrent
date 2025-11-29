import { useState, useEffect, useRef, useMemo } from 'react';
import { SpeedSample } from '../types';

interface SpeedGraphProps {
    samples: SpeedSample[];
    maxSamples?: number;
    height?: number;
    showLegend?: boolean;
    className?: string;
}

export default function SpeedGraph({ 
    samples, 
    maxSamples = 60, 
    height = 120,
    showLegend = true,
    className = ''
}: SpeedGraphProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState({ width: 0, height });

    // Calculate max values for scaling
    const { maxDownload, maxUpload, avgDownload, avgUpload } = useMemo(() => {
        if (samples.length === 0) {
            return { maxDownload: 0, maxUpload: 0, avgDownload: 0, avgUpload: 0 };
        }
        
        let maxD = 0, maxU = 0, sumD = 0, sumU = 0;
        for (const s of samples) {
            if (s.download_rate > maxD) maxD = s.download_rate;
            if (s.upload_rate > maxU) maxU = s.upload_rate;
            sumD += s.download_rate;
            sumU += s.upload_rate;
        }
        
        return {
            maxDownload: maxD,
            maxUpload: maxU,
            avgDownload: sumD / samples.length,
            avgUpload: sumU / samples.length
        };
    }, [samples]);

    // Handle resize
    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                setDimensions({ width: rect.width, height });
            }
        };

        updateDimensions();
        window.addEventListener('resize', updateDimensions);
        return () => window.removeEventListener('resize', updateDimensions);
    }, [height]);

    // Draw the graph
    useEffect(() => {
        const canvas = canvasRef.current;
        const ctx = canvas?.getContext('2d');
        if (!canvas || !ctx || dimensions.width === 0) return;

        const { width, height } = dimensions;
        const dpr = window.devicePixelRatio || 1;
        
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;
        ctx.scale(dpr, dpr);

        // Clear canvas
        ctx.clearRect(0, 0, width, height);

        // Colors
        const downloadColor = '#22d3ee'; // aurora-cyan
        const uploadColor = '#2dd4bf';   // aurora-teal
        const gridColor = 'rgba(255, 255, 255, 0.05)';
        const labelColor = 'rgba(255, 255, 255, 0.3)';

        const padding = { top: 10, right: 10, bottom: 25, left: 50 };
        const graphWidth = width - padding.left - padding.right;
        const graphHeight = height - padding.top - padding.bottom;

        // Draw background
        ctx.fillStyle = 'rgba(0, 0, 0, 0.2)';
        ctx.fillRect(padding.left, padding.top, graphWidth, graphHeight);

        // Draw grid lines
        ctx.strokeStyle = gridColor;
        ctx.lineWidth = 1;
        
        // Horizontal grid lines (4 lines)
        for (let i = 0; i <= 4; i++) {
            const y = padding.top + (graphHeight / 4) * i;
            ctx.beginPath();
            ctx.moveTo(padding.left, y);
            ctx.lineTo(width - padding.right, y);
            ctx.stroke();
        }

        // Vertical grid lines (6 lines)
        for (let i = 0; i <= 6; i++) {
            const x = padding.left + (graphWidth / 6) * i;
            ctx.beginPath();
            ctx.moveTo(x, padding.top);
            ctx.lineTo(x, height - padding.bottom);
            ctx.stroke();
        }

        // Calculate scale
        const maxValue = Math.max(maxDownload, maxUpload, 1);
        const yScale = graphHeight / maxValue;

        // Draw data
        if (samples.length > 1) {
            const xStep = graphWidth / (maxSamples - 1);
            const startIdx = Math.max(0, samples.length - maxSamples);
            const visibleSamples = samples.slice(startIdx);

            // Draw filled areas first (behind lines)
            // Download area
            ctx.fillStyle = 'rgba(34, 211, 238, 0.1)';
            ctx.beginPath();
            ctx.moveTo(padding.left, height - padding.bottom);
            visibleSamples.forEach((sample, i) => {
                const x = padding.left + i * xStep;
                const y = height - padding.bottom - sample.download_rate * yScale;
                if (i === 0) {
                    ctx.lineTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            ctx.lineTo(padding.left + (visibleSamples.length - 1) * xStep, height - padding.bottom);
            ctx.closePath();
            ctx.fill();

            // Upload area
            ctx.fillStyle = 'rgba(45, 212, 191, 0.1)';
            ctx.beginPath();
            ctx.moveTo(padding.left, height - padding.bottom);
            visibleSamples.forEach((sample, i) => {
                const x = padding.left + i * xStep;
                const y = height - padding.bottom - sample.upload_rate * yScale;
                if (i === 0) {
                    ctx.lineTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            ctx.lineTo(padding.left + (visibleSamples.length - 1) * xStep, height - padding.bottom);
            ctx.closePath();
            ctx.fill();

            // Draw lines
            // Download line
            ctx.strokeStyle = downloadColor;
            ctx.lineWidth = 2;
            ctx.beginPath();
            visibleSamples.forEach((sample, i) => {
                const x = padding.left + i * xStep;
                const y = height - padding.bottom - sample.download_rate * yScale;
                if (i === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            ctx.stroke();

            // Upload line
            ctx.strokeStyle = uploadColor;
            ctx.beginPath();
            visibleSamples.forEach((sample, i) => {
                const x = padding.left + i * xStep;
                const y = height - padding.bottom - sample.upload_rate * yScale;
                if (i === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
            });
            ctx.stroke();
        }

        // Draw Y-axis labels
        ctx.fillStyle = labelColor;
        ctx.font = '10px Inter, system-ui, sans-serif';
        ctx.textAlign = 'right';
        ctx.textBaseline = 'middle';

        for (let i = 0; i <= 4; i++) {
            const value = maxValue - (maxValue / 4) * i;
            const y = padding.top + (graphHeight / 4) * i;
            ctx.fillText(formatSpeed(value), padding.left - 5, y);
        }

        // Draw X-axis labels (time)
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        const timeLabels = ['60s', '50s', '40s', '30s', '20s', '10s', 'Now'];
        timeLabels.forEach((label, i) => {
            const x = padding.left + (graphWidth / 6) * i;
            ctx.fillText(label, x, height - padding.bottom + 5);
        });

    }, [samples, dimensions, maxDownload, maxUpload, maxSamples]);

    const formatSpeed = (bytes: number): string => {
        if (bytes === 0) return '0';
        if (bytes < 1024) return `${Math.round(bytes)} B/s`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
    };

    const currentDownload = samples.length > 0 ? samples[samples.length - 1].download_rate : 0;
    const currentUpload = samples.length > 0 ? samples[samples.length - 1].upload_rate : 0;

    return (
        <div className={`rounded-lg bg-aurora-night/30 p-4 ${className}`}>
            {showLegend && (
                <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-6">
                        <div className="flex items-center gap-2">
                            <div className="w-3 h-3 rounded-full bg-aurora-cyan" />
                            <span className="text-xs text-aurora-dim">Download</span>
                            <span className="text-sm font-medium text-aurora-cyan">
                                {formatSpeed(currentDownload)}
                            </span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-3 h-3 rounded-full bg-aurora-teal" />
                            <span className="text-xs text-aurora-dim">Upload</span>
                            <span className="text-sm font-medium text-aurora-teal">
                                {formatSpeed(currentUpload)}
                            </span>
                        </div>
                    </div>
                    <div className="flex items-center gap-4 text-xs text-aurora-dim">
                        <span>Avg ↓ {formatSpeed(avgDownload)}</span>
                        <span>Avg ↑ {formatSpeed(avgUpload)}</span>
                    </div>
                </div>
            )}
            <div ref={containerRef} className="w-full" style={{ height }}>
                <canvas ref={canvasRef} className="w-full h-full" />
            </div>
        </div>
    );
}

// Hook to collect speed samples
export function useSpeedSamples(
    downloadRate: number, 
    uploadRate: number, 
    maxSamples: number = 60
): SpeedSample[] {
    const [samples, setSamples] = useState<SpeedSample[]>([]);
    const lastUpdateRef = useRef<number>(0);

    useEffect(() => {
        const now = Date.now();
        // Only update once per second
        if (now - lastUpdateRef.current < 1000) return;
        lastUpdateRef.current = now;

        setSamples(prev => {
            const newSample: SpeedSample = {
                timestamp: now,
                download_rate: downloadRate,
                upload_rate: uploadRate
            };
            const updated = [...prev, newSample];
            // Keep only the last maxSamples
            return updated.slice(-maxSamples);
        });
    }, [downloadRate, uploadRate, maxSamples]);

    return samples;
}

