import { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
    ChevronDown, ChevronRight, 
    Download, Upload, Pause, Square, AlertCircle, CheckCircle,
    Folder, Tag, Server, Filter as FilterIcon
} from 'lucide-react';
import { Torrent, Category } from '../types';

interface FilterSidebarProps {
    torrents: Torrent[];
    categories: Record<string, Category>;
    tags: string[];
    selectedFilter: TorrentFilter;
    onFilterChange: (filter: TorrentFilter) => void;
    className?: string;
}

export interface TorrentFilter {
    status: StatusFilter;
    category: string | null; // null = all, empty string = uncategorized
    tag: string | null;      // null = all, empty string = untagged
    tracker: string | null;  // null = all, or tracker domain
}

export type StatusFilter = 
    | 'all'
    | 'downloading'
    | 'seeding'
    | 'completed'
    | 'paused'
    | 'active'
    | 'inactive'
    | 'stalled'
    | 'checking'
    | 'error';

const statusFilters: { id: StatusFilter; label: string; icon: typeof Download }[] = [
    { id: 'all', label: 'All', icon: FilterIcon },
    { id: 'downloading', label: 'Downloading', icon: Download },
    { id: 'seeding', label: 'Seeding', icon: Upload },
    { id: 'completed', label: 'Completed', icon: CheckCircle },
    { id: 'paused', label: 'Paused', icon: Pause },
    { id: 'active', label: 'Active', icon: Download },
    { id: 'inactive', label: 'Inactive', icon: Square },
    { id: 'error', label: 'Error', icon: AlertCircle },
];

export default function FilterSidebar({
    torrents,
    categories,
    tags,
    selectedFilter,
    onFilterChange,
    className = ''
}: FilterSidebarProps) {
    const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
        status: true,
        categories: true,
        tags: true,
        trackers: false,
    });

    // Count torrents by status
    const statusCounts = useMemo(() => {
        const counts: Record<StatusFilter, number> = {
            all: torrents.length,
            downloading: 0,
            seeding: 0,
            completed: 0,
            paused: 0,
            active: 0,
            inactive: 0,
            stalled: 0,
            checking: 0,
            error: 0,
        };

        for (const t of torrents) {
            const status = t.status.toLowerCase();
            
            if (status === 'downloading' || status === 'forceddownloading') {
                counts.downloading++;
                counts.active++;
            } else if (status === 'seeding' || status === 'forcedseeding') {
                counts.seeding++;
                counts.active++;
            } else if (status === 'paused') {
                counts.paused++;
                counts.inactive++;
            } else if (status === 'stopped') {
                counts.inactive++;
            } else if (status === 'checking') {
                counts.checking++;
            } else if (status === 'error') {
                counts.error++;
            }
            
            if (t.progress >= 1) {
                counts.completed++;
            }
            
            // Stalled: downloading but no speed
            if (status === 'downloading' && t.download_speed === 0) {
                counts.stalled++;
            }
        }

        return counts;
    }, [torrents]);

    // Count torrents by category
    const categoryCounts = useMemo(() => {
        const counts: Record<string, number> = { '': 0 };
        
        for (const catName of Object.keys(categories)) {
            counts[catName] = 0;
        }

        for (const t of torrents) {
            if (t.category) {
                counts[t.category] = (counts[t.category] || 0) + 1;
            } else {
                counts['']++;
            }
        }

        return counts;
    }, [torrents, categories]);

    // Count torrents by tag
    const tagCounts = useMemo(() => {
        const counts: Record<string, number> = { '': 0 };
        
        for (const tag of tags) {
            counts[tag] = 0;
        }

        for (const t of torrents) {
            if (t.tags.length === 0) {
                counts['']++;
            } else {
                for (const tag of t.tags) {
                    counts[tag] = (counts[tag] || 0) + 1;
                }
            }
        }

        return counts;
    }, [torrents, tags]);

    // Extract unique trackers
    const trackerCounts = useMemo(() => {
        const counts: Record<string, number> = {};

        for (const t of torrents) {
            const trackerDomains = new Set<string>();
            for (const tracker of t.trackers) {
                try {
                    const url = new URL(tracker.url);
                    trackerDomains.add(url.hostname);
                } catch {
                    // Invalid URL
                }
            }
            
            for (const domain of trackerDomains) {
                counts[domain] = (counts[domain] || 0) + 1;
            }
        }

        return counts;
    }, [torrents]);

    const toggleSection = (section: string) => {
        setExpandedSections(prev => ({
            ...prev,
            [section]: !prev[section]
        }));
    };

    const hasActiveFilter = 
        selectedFilter.status !== 'all' ||
        selectedFilter.category !== null ||
        selectedFilter.tag !== null ||
        selectedFilter.tracker !== null;

    return (
        <div className={`w-56 flex-shrink-0 flex flex-col overflow-hidden ${className}`}>
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-aurora-border/30">
                <h3 className="text-sm font-semibold text-aurora-text flex items-center gap-2">
                    <FilterIcon size={14} />
                    Filters
                </h3>
                {hasActiveFilter && (
                    <button
                        onClick={() => onFilterChange({
                            status: 'all',
                            category: null,
                            tag: null,
                            tracker: null
                        })}
                        className="text-xs text-aurora-dim hover:text-aurora-cyan transition-colors"
                    >
                        Clear all
                    </button>
                )}
            </div>

            <div className="flex-1 overflow-y-auto">
                {/* Status Filters */}
                <FilterSection
                    title="Status"
                    expanded={expandedSections.status}
                    onToggle={() => toggleSection('status')}
                >
                    {statusFilters.map(({ id, label, icon: Icon }) => (
                        <FilterItem
                            key={id}
                            icon={<Icon size={14} />}
                            label={label}
                            count={statusCounts[id]}
                            active={selectedFilter.status === id}
                            onClick={() => onFilterChange({ ...selectedFilter, status: id })}
                        />
                    ))}
                </FilterSection>

                {/* Category Filters */}
                <FilterSection
                    title="Categories"
                    expanded={expandedSections.categories}
                    onToggle={() => toggleSection('categories')}
                >
                    <FilterItem
                        icon={<Folder size={14} />}
                        label="All"
                        count={torrents.length}
                        active={selectedFilter.category === null}
                        onClick={() => onFilterChange({ ...selectedFilter, category: null })}
                    />
                    <FilterItem
                        icon={<Folder size={14} />}
                        label="Uncategorized"
                        count={categoryCounts['']}
                        active={selectedFilter.category === ''}
                        onClick={() => onFilterChange({ ...selectedFilter, category: '' })}
                    />
                    {Object.keys(categories).map(catName => (
                        <FilterItem
                            key={catName}
                            icon={<Folder size={14} />}
                            label={catName}
                            count={categoryCounts[catName] || 0}
                            active={selectedFilter.category === catName}
                            onClick={() => onFilterChange({ ...selectedFilter, category: catName })}
                        />
                    ))}
                </FilterSection>

                {/* Tag Filters */}
                <FilterSection
                    title="Tags"
                    expanded={expandedSections.tags}
                    onToggle={() => toggleSection('tags')}
                >
                    <FilterItem
                        icon={<Tag size={14} />}
                        label="All"
                        count={torrents.length}
                        active={selectedFilter.tag === null}
                        onClick={() => onFilterChange({ ...selectedFilter, tag: null })}
                    />
                    <FilterItem
                        icon={<Tag size={14} />}
                        label="Untagged"
                        count={tagCounts['']}
                        active={selectedFilter.tag === ''}
                        onClick={() => onFilterChange({ ...selectedFilter, tag: '' })}
                    />
                    {tags.map(tag => (
                        <FilterItem
                            key={tag}
                            icon={<Tag size={14} />}
                            label={tag}
                            count={tagCounts[tag] || 0}
                            active={selectedFilter.tag === tag}
                            onClick={() => onFilterChange({ ...selectedFilter, tag: tag })}
                        />
                    ))}
                </FilterSection>

                {/* Tracker Filters */}
                <FilterSection
                    title="Trackers"
                    expanded={expandedSections.trackers}
                    onToggle={() => toggleSection('trackers')}
                >
                    <FilterItem
                        icon={<Server size={14} />}
                        label="All"
                        count={torrents.length}
                        active={selectedFilter.tracker === null}
                        onClick={() => onFilterChange({ ...selectedFilter, tracker: null })}
                    />
                    {Object.entries(trackerCounts)
                        .sort((a, b) => b[1] - a[1])
                        .map(([domain, count]) => (
                            <FilterItem
                                key={domain}
                                icon={<Server size={14} />}
                                label={domain}
                                count={count}
                                active={selectedFilter.tracker === domain}
                                onClick={() => onFilterChange({ ...selectedFilter, tracker: domain })}
                            />
                        ))
                    }
                </FilterSection>
            </div>
        </div>
    );
}

// =============================================================================
// Sub-components
// =============================================================================

function FilterSection({ 
    title, 
    expanded, 
    onToggle, 
    children 
}: { 
    title: string; 
    expanded: boolean; 
    onToggle: () => void; 
    children: React.ReactNode;
}) {
    return (
        <div className="border-b border-aurora-border/20">
            <button
                onClick={onToggle}
                className="w-full flex items-center gap-2 px-4 py-2 text-sm font-medium text-aurora-dim hover:text-aurora-text transition-colors"
            >
                {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                {title}
            </button>
            <AnimatePresence>
                {expanded && (
                    <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={{ duration: 0.2 }}
                        className="overflow-hidden"
                    >
                        <div className="pb-2">
                            {children}
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}

function FilterItem({
    icon,
    label,
    count,
    active,
    onClick
}: {
    icon: React.ReactNode;
    label: string;
    count: number;
    active: boolean;
    onClick: () => void;
}) {
    return (
        <button
            onClick={onClick}
            className={`w-full flex items-center gap-2 px-4 py-1.5 text-sm transition-colors ${
                active
                    ? 'text-aurora-cyan bg-aurora-cyan/10'
                    : 'text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/30'
            }`}
        >
            <span className={active ? 'text-aurora-cyan' : 'text-aurora-dim'}>
                {icon}
            </span>
            <span className="flex-1 text-left truncate">{label}</span>
            <span className={`text-xs px-1.5 py-0.5 rounded ${
                active ? 'bg-aurora-cyan/20 text-aurora-cyan' : 'bg-aurora-night/50 text-aurora-dim'
            }`}>
                {count}
            </span>
        </button>
    );
}

// =============================================================================
// Filter Logic
// =============================================================================

export function filterTorrents(torrents: Torrent[], filter: TorrentFilter): Torrent[] {
    return torrents.filter(t => {
        // Status filter
        if (filter.status !== 'all') {
            const status = t.status.toLowerCase();
            
            switch (filter.status) {
                case 'downloading':
                    if (status !== 'downloading' && status !== 'forceddownloading') return false;
                    break;
                case 'seeding':
                    if (status !== 'seeding' && status !== 'forcedseeding') return false;
                    break;
                case 'completed':
                    if (t.progress < 1) return false;
                    break;
                case 'paused':
                    if (status !== 'paused') return false;
                    break;
                case 'active':
                    if (t.download_speed === 0 && t.upload_speed === 0) return false;
                    break;
                case 'inactive':
                    if (t.download_speed > 0 || t.upload_speed > 0) return false;
                    break;
                case 'stalled':
                    if (status !== 'downloading' || t.download_speed > 0) return false;
                    break;
                case 'checking':
                    if (status !== 'checking') return false;
                    break;
                case 'error':
                    if (status !== 'error') return false;
                    break;
            }
        }

        // Category filter
        if (filter.category !== null) {
            if (filter.category === '') {
                // Uncategorized
                if (t.category) return false;
            } else {
                if (t.category !== filter.category) return false;
            }
        }

        // Tag filter
        if (filter.tag !== null) {
            if (filter.tag === '') {
                // Untagged
                if (t.tags.length > 0) return false;
            } else {
                if (!t.tags.includes(filter.tag)) return false;
            }
        }

        // Tracker filter
        if (filter.tracker !== null) {
            const hasTracker = t.trackers.some(tracker => {
                try {
                    const url = new URL(tracker.url);
                    return url.hostname === filter.tracker;
                } catch {
                    return false;
                }
            });
            if (!hasTracker) return false;
        }

        return true;
    });
}

// Default filter state
export const defaultFilter: TorrentFilter = {
    status: 'all',
    category: null,
    tag: null,
    tracker: null
};

