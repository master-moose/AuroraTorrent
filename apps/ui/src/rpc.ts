import { invoke } from '@tauri-apps/api/tauri';
import {
    Torrent,
    FileInfo,
    FilePriority,
    Config,
    Category,
    RssFeed,
    RssArticle,
    RssDownloadRule,
    SearchResult,
    SearchPlugin,
    SessionStats,
    TorrentLimits,
    AddTorrentParams,
} from './types';

// =============================================================================
// TORRENT MANAGEMENT
// =============================================================================

export async function listTorrents(): Promise<Torrent[]> {
    try {
        return await invoke('list_torrents');
    } catch (e) {
        console.error('Failed to list torrents:', e);
        return [];
    }
}

export async function addTorrent(
    magnet: string,
    params?: AddTorrentParams
): Promise<{ name?: string; id?: string; error?: string }> {
    try {
        const result = await invoke('add_torrent', { magnet, params });
        return result as { name?: string; id?: string };
    } catch (e) {
        return { error: String(e) };
    }
}

export async function addTorrentFile(
    name: string,
    content: string,
    params?: AddTorrentParams
): Promise<{ name?: string; id?: string; error?: string }> {
    try {
        const result = await invoke('add_torrent_file', { name, content, params });
        return result as { name?: string; id?: string };
    } catch (e) {
        return { error: String(e) };
    }
}

export async function startTorrent(id: string): Promise<{ error?: string }> {
    try {
        await invoke('start_torrent', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function pauseTorrent(id: string): Promise<{ error?: string }> {
    try {
        await invoke('pause_torrent', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function stopTorrent(id: string): Promise<{ error?: string }> {
    try {
        await invoke('stop_torrent', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function removeTorrent(
    id: string,
    deleteFiles: boolean = false
): Promise<{ error?: string }> {
    try {
        await invoke('remove_torrent', { id, deleteFiles });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function forceRecheck(id: string): Promise<{ error?: string }> {
    try {
        await invoke('force_recheck', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function forceReannounce(id: string): Promise<{ error?: string }> {
    try {
        await invoke('force_reannounce', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function renameTorrent(
    id: string,
    newName: string
): Promise<{ error?: string }> {
    try {
        await invoke('rename_torrent', { id, newName });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function setTorrentCategory(
    id: string,
    category: string | null
): Promise<{ error?: string }> {
    try {
        await invoke('set_torrent_category', { id, category });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function addTorrentTags(
    id: string,
    tags: string[]
): Promise<{ error?: string }> {
    try {
        await invoke('add_torrent_tags', { id, tags });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function removeTorrentTags(
    id: string,
    tags: string[]
): Promise<{ error?: string }> {
    try {
        await invoke('remove_torrent_tags', { id, tags });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function setTorrentLimits(
    id: string,
    limits: TorrentLimits
): Promise<{ error?: string }> {
    try {
        await invoke('set_torrent_limits', { id, limits });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function toggleSequentialDownload(
    id: string,
    enabled: boolean
): Promise<{ error?: string }> {
    try {
        await invoke('toggle_sequential_download', { id, enabled });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function toggleFirstLastPiecePriority(
    id: string,
    enabled: boolean
): Promise<{ error?: string }> {
    try {
        await invoke('toggle_first_last_piece_priority', { id, enabled });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// STREAMING
// =============================================================================

export async function streamTorrent(
    id: string
): Promise<{ url?: string; error?: string }> {
    try {
        const result = await invoke('stream_torrent', { id });
        return result as { url?: string };
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// TRACKERS
// =============================================================================

export async function addTrackers(
    id: string,
    trackers: string[]
): Promise<{ error?: string }> {
    try {
        await invoke('add_trackers', { id, trackers });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function removeTrackers(
    id: string,
    trackers: string[]
): Promise<{ error?: string }> {
    try {
        await invoke('remove_trackers', { id, trackers });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// FILE PRIORITY MANAGEMENT
// =============================================================================

export async function getTorrentFiles(id: string): Promise<FileInfo[]> {
    try {
        return await invoke('get_torrent_files', { id });
    } catch (e) {
        console.error('Failed to get torrent files:', e);
        return [];
    }
}

export async function setFilePriority(
    torrentId: string,
    fileIndex: number,
    priority: FilePriority
): Promise<{ error?: string }> {
    try {
        await invoke('set_file_priority', { torrentId, fileIndex, priority });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function setFilesPriority(
    torrentId: string,
    fileIndices: number[],
    priority: FilePriority
): Promise<{ error?: string }> {
    try {
        await invoke('set_files_priority', { torrentId, fileIndices, priority });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// QUEUE MANAGEMENT
// =============================================================================

export async function queueMoveUp(id: string): Promise<{ error?: string }> {
    try {
        await invoke('queue_move_up', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function queueMoveDown(id: string): Promise<{ error?: string }> {
    try {
        await invoke('queue_move_down', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function queueMoveTop(id: string): Promise<{ error?: string }> {
    try {
        await invoke('queue_move_top', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function queueMoveBottom(id: string): Promise<{ error?: string }> {
    try {
        await invoke('queue_move_bottom', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// CONFIGURATION
// =============================================================================

export async function getConfig(): Promise<Config | null> {
    try {
        return await invoke('get_config');
    } catch (e) {
        console.error('Failed to get config:', e);
        return null;
    }
}

export async function setConfig(config: Partial<Config>): Promise<{ error?: string }> {
    try {
        await invoke('set_config', { config });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function toggleAltSpeed(enabled: boolean): Promise<{ error?: string }> {
    try {
        await invoke('toggle_alt_speed', { enabled });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// CATEGORIES AND TAGS
// =============================================================================

export async function createCategory(
    name: string,
    savePath?: string
): Promise<{ error?: string }> {
    try {
        await invoke('create_category', { name, savePath });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function editCategory(
    name: string,
    savePath?: string
): Promise<{ error?: string }> {
    try {
        await invoke('edit_category', { name, savePath });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function deleteCategory(name: string): Promise<{ error?: string }> {
    try {
        await invoke('delete_category', { name });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function getCategories(): Promise<Record<string, Category>> {
    try {
        return await invoke('get_categories');
    } catch (e) {
        console.error('Failed to get categories:', e);
        return {};
    }
}

export async function createTag(tag: string): Promise<{ error?: string }> {
    try {
        await invoke('create_tag', { tag });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function deleteTag(tag: string): Promise<{ error?: string }> {
    try {
        await invoke('delete_tag', { tag });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function getTags(): Promise<string[]> {
    try {
        return await invoke('get_tags');
    } catch (e) {
        console.error('Failed to get tags:', e);
        return [];
    }
}

// =============================================================================
// RSS FEEDS
// =============================================================================

export async function getRssFeeds(): Promise<RssFeed[]> {
    try {
        return await invoke('get_rss_feeds');
    } catch (e) {
        console.error('Failed to get RSS feeds:', e);
        return [];
    }
}

export async function addRssFeed(
    url: string,
    name?: string
): Promise<{ feed?: RssFeed; error?: string }> {
    try {
        const feed = await invoke<RssFeed>('add_rss_feed', { url, name });
        return { feed };
    } catch (e) {
        return { error: String(e) };
    }
}

export async function removeRssFeed(id: string): Promise<{ error?: string }> {
    try {
        await invoke('remove_rss_feed', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function refreshRssFeed(id: string): Promise<{ error?: string }> {
    try {
        await invoke('refresh_rss_feed', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function getRssArticles(feedId?: string): Promise<RssArticle[]> {
    try {
        return await invoke('get_rss_articles', { feedId });
    } catch (e) {
        console.error('Failed to get RSS articles:', e);
        return [];
    }
}

export async function getRssRules(): Promise<RssDownloadRule[]> {
    try {
        return await invoke('get_rss_rules');
    } catch (e) {
        console.error('Failed to get RSS rules:', e);
        return [];
    }
}

export async function addRssRule(rule: RssDownloadRule): Promise<{ error?: string }> {
    try {
        await invoke('add_rss_rule', { rule });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function removeRssRule(id: string): Promise<{ error?: string }> {
    try {
        await invoke('remove_rss_rule', { id });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// SEARCH
// =============================================================================

export async function searchTorrents(
    query: string,
    plugins?: string[],
    category?: string
): Promise<SearchResult[]> {
    try {
        return await invoke('search_torrents', { query, plugins, category });
    } catch (e) {
        console.error('Search failed:', e);
        return [];
    }
}

export async function getSearchPlugins(): Promise<SearchPlugin[]> {
    try {
        return await invoke('get_search_plugins');
    } catch (e) {
        console.error('Failed to get search plugins:', e);
        return [];
    }
}

export async function enableSearchPlugin(
    name: string,
    enabled: boolean
): Promise<{ error?: string }> {
    try {
        await invoke('enable_search_plugin', { name, enabled });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// TORRENT CREATION
// =============================================================================

export async function createTorrent(
    sourcePath: string,
    trackers: string[],
    comment?: string,
    isPrivate: boolean = false,
    pieceSize?: number
): Promise<{ data?: Uint8Array; error?: string }> {
    try {
        const data = await invoke<number[]>('create_torrent', {
            sourcePath,
            trackers,
            comment,
            isPrivate,
            pieceSize,
        });
        return { data: new Uint8Array(data) };
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// SESSION STATS
// =============================================================================

export async function getSessionStats(): Promise<SessionStats | null> {
    try {
        return await invoke('get_session_stats');
    } catch (e) {
        console.error('Failed to get session stats:', e);
        return null;
    }
}

// =============================================================================
// UTILITIES
// =============================================================================

export async function openFolder(path: string): Promise<{ error?: string }> {
    try {
        await invoke('open_folder', { path });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

export async function banPeer(ip: string): Promise<{ error?: string }> {
    try {
        await invoke('ban_peer', { ip });
        return {};
    } catch (e) {
        return { error: String(e) };
    }
}

// =============================================================================
// LEGACY COMPATIBILITY LAYER
// =============================================================================

export const sendRpc = async (
    method: string,
    params: Record<string, unknown> = {}
): Promise<{
    result?: unknown;
    error?: string;
}> => {
    try {
        switch (method) {
            case 'ListTorrents': {
                const result = await listTorrents();
                return { result };
            }
            case 'AddTorrent': {
                const result = await addTorrent(params.magnet as string);
                if (result.error) return { error: result.error };
                return { result };
            }
            case 'AddTorrentFile': {
                const result = await addTorrentFile(
                    params.name as string,
                    params.content as string
                );
                if (result.error) return { error: result.error };
                return { result };
            }
            case 'StartTorrent': {
                const result = await startTorrent(params.id as string);
                if (result.error) return { error: result.error };
                return { result: { status: 'started' } };
            }
            case 'PauseTorrent': {
                const result = await pauseTorrent(params.id as string);
                if (result.error) return { error: result.error };
                return { result: { status: 'paused' } };
            }
            case 'StopTorrent': {
                const result = await stopTorrent(params.id as string);
                if (result.error) return { error: result.error };
                return { result: { status: 'stopped' } };
            }
            case 'RemoveTorrent': {
                const result = await removeTorrent(
                    params.id as string,
                    params.deleteFiles as boolean | undefined
                );
                if (result.error) return { error: result.error };
                return { result: { status: 'removed' } };
            }
            case 'StreamTorrent': {
                const result = await streamTorrent(params.id as string);
                if (result.error) return { error: result.error };
                return { result };
            }
            case 'GetConfig': {
                const result = await getConfig();
                return { result };
            }
            case 'SetConfig': {
                const result = await setConfig(params as Partial<Config>);
                if (result.error) return { error: result.error };
                return { result: { status: 'updated' } };
            }
            default:
                return { error: `Unknown method: ${method}` };
        }
    } catch (e) {
        console.error('RPC Error:', e);
        return { error: String(e) };
    }
};
