import { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { 
    X, Save, Folder, ArrowDown, ArrowUp,
    Wifi, Zap, Globe, Settings2, 
    Plus, Trash2, Check, ChevronRight
} from 'lucide-react';
import { getConfig, setConfig } from '../rpc';
import { 
    Config, EncryptionMode, ShareLimitAction, ProxyType
} from '../types';

interface SettingsModalProps {
    onClose: () => void;
}

type TabId = 'downloads' | 'connection' | 'speed' | 'bittorrent' | 'webui' | 'advanced';

const tabs: { id: TabId; label: string; icon: typeof Folder }[] = [
    { id: 'downloads', label: 'Downloads', icon: Folder },
    { id: 'connection', label: 'Connection', icon: Wifi },
    { id: 'speed', label: 'Speed', icon: Zap },
    { id: 'bittorrent', label: 'BitTorrent', icon: Globe },
    { id: 'webui', label: 'Web UI', icon: Globe },
    { id: 'advanced', label: 'Advanced', icon: Settings2 },
];

export default function SettingsModal({ onClose }: SettingsModalProps) {
    const [config, setLocalConfig] = useState<Partial<Config>>({});
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);
    const [activeTab, setActiveTab] = useState<TabId>('downloads');
    const [hasChanges, setHasChanges] = useState(false);
    const [newTag, setNewTag] = useState('');
    const [newCategory, setNewCategory] = useState({ name: '', save_path: '' });
    const [newBannedIp, setNewBannedIp] = useState('');

    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const loadedConfig = await getConfig() as Config | null;
                if (loadedConfig) {
                    setLocalConfig(loadedConfig);
                }
            } catch (error) {
                console.error('Failed to fetch settings:', error);
            } finally {
                setLoading(false);
            }
        };
        fetchConfig();
    }, []);

    const updateConfig = <K extends keyof Config>(key: K, value: Config[K]) => {
        setLocalConfig(prev => ({ ...prev, [key]: value }));
        setHasChanges(true);
    };

    const updateNestedConfig = <K extends keyof Config>(
        section: K, 
        updates: Partial<Config[K]>
    ) => {
        setLocalConfig(prev => ({
            ...prev,
            [section]: { ...(prev[section] as object), ...updates }
        }));
        setHasChanges(true);
    };

    const handleSave = async () => {
        setSaving(true);
        try {
            await setConfig(config);
            setHasChanges(false);
            onClose();
        } catch (error) {
            console.error('Failed to save settings:', error);
        } finally {
            setSaving(false);
        }
    };

    const formatSpeedLabel = (bytes: number) => {
        if (bytes === 0) return 'Unlimited';
        if (bytes < 1024) return `${bytes} B/s`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
    };

    const addTag = () => {
        if (newTag && !config.tags?.includes(newTag)) {
            updateConfig('tags', [...(config.tags || []), newTag]);
            setNewTag('');
        }
    };

    const removeTag = (tag: string) => {
        updateConfig('tags', (config.tags || []).filter(t => t !== tag));
    };

    const addCategory = () => {
        if (newCategory.name) {
            const categories = { ...(config.categories || {}) };
            categories[newCategory.name] = { 
                name: newCategory.name, 
                save_path: newCategory.save_path || undefined 
            };
            updateConfig('categories', categories);
            setNewCategory({ name: '', save_path: '' });
        }
    };

    const removeCategory = (name: string) => {
        const categories = { ...(config.categories || {}) };
        delete categories[name];
        updateConfig('categories', categories);
    };

    const addBannedIp = () => {
        if (newBannedIp && !config.ip_filter?.banned_ips?.includes(newBannedIp)) {
            updateNestedConfig('ip_filter', {
                banned_ips: [...(config.ip_filter?.banned_ips || []), newBannedIp]
            });
            setNewBannedIp('');
        }
    };

    const removeBannedIp = (ip: string) => {
        updateNestedConfig('ip_filter', {
            banned_ips: (config.ip_filter?.banned_ips || []).filter(i => i !== ip)
        });
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
                className="card w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden"
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-6 border-b border-aurora-border/50">
                    <div className="flex items-center gap-3">
                        <Settings2 className="w-6 h-6 text-aurora-cyan" />
                        <h2 className="text-xl font-bold text-aurora-text">Settings</h2>
                        {hasChanges && (
                            <span className="px-2 py-1 text-xs rounded-full bg-aurora-violet/20 text-aurora-violet">
                                Unsaved changes
                            </span>
                        )}
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 rounded-lg transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                <div className="flex flex-1 overflow-hidden">
                    {/* Sidebar Tabs */}
                    <div className="w-48 border-r border-aurora-border/30 p-2 overflow-y-auto flex-shrink-0">
                        {tabs.map(({ id, label, icon: Icon }) => (
                            <button
                                key={id}
                                onClick={() => setActiveTab(id)}
                                className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left text-sm font-medium transition-colors mb-1 ${
                                    activeTab === id
                                        ? 'bg-aurora-cyan/10 text-aurora-cyan'
                                        : 'text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/30'
                                }`}
                            >
                                <Icon size={18} />
                                {label}
                                {activeTab === id && <ChevronRight size={16} className="ml-auto" />}
                            </button>
                        ))}
                    </div>

                    {/* Content */}
                    <div className="flex-1 overflow-y-auto p-6">
                        {/* Downloads Tab */}
                        {activeTab === 'downloads' && (
                            <div className="space-y-6">
                                <Section title="Save Location">
                                    <div className="space-y-4">
                                        <InputField
                                            label="Default Save Path"
                                            icon={<Folder size={16} />}
                                            value={config.download_path || ''}
                                            onChange={(v) => updateConfig('download_path', v)}
                                            placeholder="/path/to/downloads"
                                        />
                                        <div className="flex items-center gap-3">
                                            <input
                                                type="checkbox"
                                                checked={config.use_temp_path || false}
                                                onChange={(e) => updateConfig('use_temp_path', e.target.checked)}
                                                className="w-4 h-4 rounded accent-aurora-cyan"
                                            />
                                            <label className="text-sm text-aurora-text">Use temporary folder for incomplete downloads</label>
                                        </div>
                                        {config.use_temp_path && (
                                            <InputField
                                                label="Temporary Path"
                                                value={config.temp_path || ''}
                                                onChange={(v) => updateConfig('temp_path', v)}
                                                placeholder="/path/to/temp"
                                            />
                                        )}
                                    </div>
                                </Section>

                                <Section title="Categories">
                                    <div className="space-y-3">
                                        {Object.values(config.categories || {}).map((cat) => (
                                            <div key={cat.name} className="flex items-center gap-3 p-3 rounded-lg bg-aurora-night/30">
                                                <Folder size={16} className="text-aurora-cyan" />
                                                <div className="flex-1">
                                                    <p className="text-sm text-aurora-text">{cat.name}</p>
                                                    {cat.save_path && (
                                                        <p className="text-xs text-aurora-dim">{cat.save_path}</p>
                                                    )}
                                                </div>
                                                <button
                                                    onClick={() => removeCategory(cat.name)}
                                                    className="p-1 text-aurora-dim hover:text-aurora-rose transition-colors"
                                                >
                                                    <Trash2 size={14} />
                                                </button>
                                            </div>
                                        ))}
                                        <div className="flex gap-2">
                                            <input
                                                type="text"
                                                value={newCategory.name}
                                                onChange={(e) => setNewCategory(prev => ({ ...prev, name: e.target.value }))}
                                                placeholder="Category name"
                                                className="input flex-1"
                                            />
                                            <input
                                                type="text"
                                                value={newCategory.save_path}
                                                onChange={(e) => setNewCategory(prev => ({ ...prev, save_path: e.target.value }))}
                                                placeholder="Save path (optional)"
                                                className="input flex-1"
                                            />
                                            <button onClick={addCategory} className="btn-secondary px-3">
                                                <Plus size={16} />
                                            </button>
                                        </div>
                                    </div>
                                </Section>

                                <Section title="Tags">
                                    <div className="space-y-3">
                                        <div className="flex flex-wrap gap-2">
                                            {(config.tags || []).map((tag) => (
                                                <span 
                                                    key={tag} 
                                                    className="inline-flex items-center gap-1 px-3 py-1 rounded-full bg-aurora-violet/20 text-aurora-violet text-sm"
                                                >
                                                    {tag}
                                                    <button
                                                        onClick={() => removeTag(tag)}
                                                        className="hover:text-aurora-rose transition-colors"
                                                    >
                                                        <X size={12} />
                                                    </button>
                                                </span>
                                            ))}
                                        </div>
                                        <div className="flex gap-2">
                                            <input
                                                type="text"
                                                value={newTag}
                                                onChange={(e) => setNewTag(e.target.value)}
                                                onKeyDown={(e) => e.key === 'Enter' && addTag()}
                                                placeholder="Add tag..."
                                                className="input flex-1"
                                            />
                                            <button onClick={addTag} className="btn-secondary px-3">
                                                <Plus size={16} />
                                            </button>
                                        </div>
                                    </div>
                                </Section>

                                <Section title="Options">
                                    <div className="space-y-3">
                                        <Checkbox
                                            label="Pre-allocate disk space for all files"
                                            checked={config.preallocate_all || false}
                                            onChange={(v) => updateConfig('preallocate_all', v)}
                                        />
                                        <Checkbox
                                            label="Append .!at extension to incomplete files"
                                            checked={config.incomplete_extension || false}
                                            onChange={(v) => updateConfig('incomplete_extension', v)}
                                        />
                                        <Checkbox
                                            label="Start torrents automatically when added"
                                            checked={config.auto_start !== false}
                                            onChange={(v) => updateConfig('auto_start', v)}
                                        />
                                        <Checkbox
                                            label="Delete .torrent files after adding"
                                            checked={config.delete_torrent_files || false}
                                            onChange={(v) => updateConfig('delete_torrent_files', v)}
                                        />
                                    </div>
                                </Section>
                            </div>
                        )}

                        {/* Connection Tab */}
                        {activeTab === 'connection' && (
                            <div className="space-y-6">
                                <Section title="Listening Port">
                                    <div className="space-y-4">
                                        <div className="grid grid-cols-2 gap-4">
                                            <InputField
                                                label="Port"
                                                type="number"
                                                value={config.connection?.listen_port?.toString() || '6881'}
                                                onChange={(v) => updateNestedConfig('connection', { listen_port: parseInt(v) || 6881 })}
                                            />
                                        </div>
                                        <Checkbox
                                            label="Use UPnP / NAT-PMP for port forwarding"
                                            checked={config.connection?.upnp_enabled !== false}
                                            onChange={(v) => updateNestedConfig('connection', { upnp_enabled: v })}
                                        />
                                        <Checkbox
                                            label="Use random port on startup"
                                            checked={config.connection?.random_port || false}
                                            onChange={(v) => updateNestedConfig('connection', { random_port: v })}
                                        />
                                    </div>
                                </Section>

                                <Section title="Connection Limits">
                                    <div className="grid grid-cols-2 gap-4">
                                        <InputField
                                            label="Global max connections"
                                            type="number"
                                            value={config.connection?.max_connections?.toString() || '500'}
                                            onChange={(v) => updateNestedConfig('connection', { max_connections: parseInt(v) || 500 })}
                                        />
                                        <InputField
                                            label="Max connections per torrent"
                                            type="number"
                                            value={config.connection?.max_connections_per_torrent?.toString() || '100'}
                                            onChange={(v) => updateNestedConfig('connection', { max_connections_per_torrent: parseInt(v) || 100 })}
                                        />
                                        <InputField
                                            label="Global max upload slots"
                                            type="number"
                                            value={config.connection?.max_uploads?.toString() || '20'}
                                            onChange={(v) => updateNestedConfig('connection', { max_uploads: parseInt(v) || 20 })}
                                        />
                                        <InputField
                                            label="Max upload slots per torrent"
                                            type="number"
                                            value={config.connection?.max_uploads_per_torrent?.toString() || '4'}
                                            onChange={(v) => updateNestedConfig('connection', { max_uploads_per_torrent: parseInt(v) || 4 })}
                                        />
                                    </div>
                                </Section>

                                <Section title="Proxy Server">
                                    <div className="space-y-4">
                                        <Checkbox
                                            label="Enable proxy"
                                            checked={config.proxy?.enabled || false}
                                            onChange={(v) => updateNestedConfig('proxy', { enabled: v })}
                                        />
                                        {config.proxy?.enabled && (
                                            <>
                                                <div className="grid grid-cols-3 gap-4">
                                                    <div>
                                                        <label className="text-sm text-aurora-dim mb-1 block">Type</label>
                                                        <select
                                                            value={config.proxy?.proxy_type || 'None'}
                                                            onChange={(e) => updateNestedConfig('proxy', { proxy_type: e.target.value as ProxyType })}
                                                            className="input"
                                                        >
                                                            <option value="Http">HTTP</option>
                                                            <option value="Socks4">SOCKS4</option>
                                                            <option value="Socks5">SOCKS5</option>
                                                            <option value="Socks5WithAuth">SOCKS5 (Auth)</option>
                                                            <option value="HttpWithAuth">HTTP (Auth)</option>
                                                        </select>
                                                    </div>
                                                    <InputField
                                                        label="Host"
                                                        value={config.proxy?.host || ''}
                                                        onChange={(v) => updateNestedConfig('proxy', { host: v })}
                                                    />
                                                    <InputField
                                                        label="Port"
                                                        type="number"
                                                        value={config.proxy?.port?.toString() || ''}
                                                        onChange={(v) => updateNestedConfig('proxy', { port: parseInt(v) || 0 })}
                                                    />
                                                </div>
                                                {(config.proxy?.proxy_type === 'Socks5WithAuth' || config.proxy?.proxy_type === 'HttpWithAuth') && (
                                                    <div className="grid grid-cols-2 gap-4">
                                                        <InputField
                                                            label="Username"
                                                            value={config.proxy?.username || ''}
                                                            onChange={(v) => updateNestedConfig('proxy', { username: v })}
                                                        />
                                                        <InputField
                                                            label="Password"
                                                            type="password"
                                                            value={config.proxy?.password || ''}
                                                            onChange={(v) => updateNestedConfig('proxy', { password: v })}
                                                        />
                                                    </div>
                                                )}
                                                <div className="space-y-2">
                                                    <Checkbox
                                                        label="Use for peer connections"
                                                        checked={config.proxy?.use_for_peer_connections || false}
                                                        onChange={(v) => updateNestedConfig('proxy', { use_for_peer_connections: v })}
                                                    />
                                                    <Checkbox
                                                        label="Use for tracker connections"
                                                        checked={config.proxy?.use_for_tracker_connections || false}
                                                        onChange={(v) => updateNestedConfig('proxy', { use_for_tracker_connections: v })}
                                                    />
                                                </div>
                                            </>
                                        )}
                                    </div>
                                </Section>

                                <Section title="IP Filter">
                                    <div className="space-y-4">
                                        <Checkbox
                                            label="Enable IP filtering"
                                            checked={config.ip_filter?.enabled || false}
                                            onChange={(v) => updateNestedConfig('ip_filter', { enabled: v })}
                                        />
                                        {config.ip_filter?.enabled && (
                                            <>
                                                <InputField
                                                    label="Filter file path (DAT/P2P format)"
                                                    value={config.ip_filter?.filter_path || ''}
                                                    onChange={(v) => updateNestedConfig('ip_filter', { filter_path: v })}
                                                    placeholder="/path/to/filter.dat"
                                                />
                                                <div>
                                                    <label className="text-sm text-aurora-dim mb-2 block">Banned IPs</label>
                                                    <div className="space-y-2 max-h-32 overflow-y-auto mb-2">
                                                        {(config.ip_filter?.banned_ips || []).map((ip) => (
                                                            <div key={ip} className="flex items-center gap-2 p-2 rounded bg-aurora-night/30">
                                                                <span className="text-sm font-mono text-aurora-text flex-1">{ip}</span>
                                                                <button
                                                                    onClick={() => removeBannedIp(ip)}
                                                                    className="text-aurora-dim hover:text-aurora-rose"
                                                                >
                                                                    <Trash2 size={14} />
                                                                </button>
                                                            </div>
                                                        ))}
                                                    </div>
                                                    <div className="flex gap-2">
                                                        <input
                                                            type="text"
                                                            value={newBannedIp}
                                                            onChange={(e) => setNewBannedIp(e.target.value)}
                                                            placeholder="IP address to ban"
                                                            className="input flex-1"
                                                        />
                                                        <button onClick={addBannedIp} className="btn-secondary px-3">
                                                            <Plus size={16} />
                                                        </button>
                                                    </div>
                                                </div>
                                            </>
                                        )}
                                    </div>
                                </Section>
                            </div>
                        )}

                        {/* Speed Tab */}
                        {activeTab === 'speed' && (
                            <div className="space-y-6">
                                <Section title="Global Speed Limits">
                                    <div className="grid grid-cols-2 gap-4">
                                        <div>
                                            <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                                                <ArrowDown size={16} className="text-aurora-cyan" />
                                                Download Limit
                                            </label>
                                            <input
                                                type="number"
                                                min="0"
                                                step="102400"
                                                value={config.max_download_speed || 0}
                                                onChange={(e) => updateConfig('max_download_speed', Math.max(0, Number(e.target.value)))}
                                                className="input"
                                            />
                                            <p className="text-xs text-aurora-muted mt-1">
                                                {formatSpeedLabel(config.max_download_speed || 0)}
                                            </p>
                                        </div>
                                        <div>
                                            <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                                                <ArrowUp size={16} className="text-aurora-teal" />
                                                Upload Limit
                                            </label>
                                            <input
                                                type="number"
                                                min="0"
                                                step="102400"
                                                value={config.max_upload_speed || 0}
                                                onChange={(e) => updateConfig('max_upload_speed', Math.max(0, Number(e.target.value)))}
                                                className="input"
                                            />
                                            <p className="text-xs text-aurora-muted mt-1">
                                                {formatSpeedLabel(config.max_upload_speed || 0)}
                                            </p>
                                        </div>
                                    </div>
                                    <p className="text-xs text-aurora-muted mt-2">Set to 0 for unlimited speed. Values are in bytes per second.</p>
                                </Section>

                                <Section title="Alternative Speed Limits">
                                    <div className="space-y-4">
                                        <Checkbox
                                            label="Enable alternative speed limits"
                                            checked={config.use_alt_speed_limits || false}
                                            onChange={(v) => updateConfig('use_alt_speed_limits', v)}
                                        />
                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="text-sm font-medium text-aurora-dim mb-2 block">Alt Download Limit</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    step="102400"
                                                    value={config.alt_download_speed || 0}
                                                    onChange={(e) => updateConfig('alt_download_speed', Math.max(0, Number(e.target.value)))}
                                                    className="input"
                                                />
                                                <p className="text-xs text-aurora-muted mt-1">
                                                    {formatSpeedLabel(config.alt_download_speed || 0)}
                                                </p>
                                            </div>
                                            <div>
                                                <label className="text-sm font-medium text-aurora-dim mb-2 block">Alt Upload Limit</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    step="102400"
                                                    value={config.alt_upload_speed || 0}
                                                    onChange={(e) => updateConfig('alt_upload_speed', Math.max(0, Number(e.target.value)))}
                                                    className="input"
                                                />
                                                <p className="text-xs text-aurora-muted mt-1">
                                                    {formatSpeedLabel(config.alt_upload_speed || 0)}
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </Section>

                                <Section title="Queue Settings">
                                    <div className="space-y-4">
                                        <div className="grid grid-cols-3 gap-4">
                                            <InputField
                                                label="Max active downloads"
                                                type="number"
                                                value={config.queue?.max_active_downloads?.toString() || '3'}
                                                onChange={(v) => updateNestedConfig('queue', { max_active_downloads: Math.max(1, parseInt(v) || 3) })}
                                            />
                                            <InputField
                                                label="Max active uploads"
                                                type="number"
                                                value={config.queue?.max_active_uploads?.toString() || '5'}
                                                onChange={(v) => updateNestedConfig('queue', { max_active_uploads: Math.max(1, parseInt(v) || 5) })}
                                            />
                                            <InputField
                                                label="Max active total"
                                                type="number"
                                                value={config.queue?.max_active_torrents?.toString() || '5'}
                                                onChange={(v) => updateNestedConfig('queue', { max_active_torrents: Math.max(1, parseInt(v) || 5) })}
                                            />
                                        </div>
                                        <Checkbox
                                            label="Do not count slow torrents in queue limits"
                                            checked={config.queue?.ignore_slow_torrents || false}
                                            onChange={(v) => updateNestedConfig('queue', { ignore_slow_torrents: v })}
                                        />
                                    </div>
                                </Section>
                            </div>
                        )}

                        {/* BitTorrent Tab */}
                        {activeTab === 'bittorrent' && (
                            <div className="space-y-6">
                                <Section title="Privacy">
                                    <div className="space-y-3">
                                        <Checkbox
                                            label="Enable DHT (decentralized network) to find more peers"
                                            checked={config.bittorrent?.dht_enabled !== false}
                                            onChange={(v) => updateNestedConfig('bittorrent', { dht_enabled: v })}
                                        />
                                        <Checkbox
                                            label="Enable Peer Exchange (PeX) to find more peers"
                                            checked={config.bittorrent?.pex_enabled !== false}
                                            onChange={(v) => updateNestedConfig('bittorrent', { pex_enabled: v })}
                                        />
                                        <Checkbox
                                            label="Enable Local Peer Discovery"
                                            checked={config.bittorrent?.lsd_enabled !== false}
                                            onChange={(v) => updateNestedConfig('bittorrent', { lsd_enabled: v })}
                                        />
                                        <Checkbox
                                            label="Enable anonymous mode (no identifying information sent)"
                                            checked={config.bittorrent?.anonymous_mode || false}
                                            onChange={(v) => updateNestedConfig('bittorrent', { anonymous_mode: v })}
                                        />
                                    </div>
                                </Section>

                                <Section title="Encryption">
                                    <div>
                                        <label className="text-sm text-aurora-dim mb-2 block">Encryption mode</label>
                                        <select
                                            value={config.bittorrent?.encryption || 'Prefer'}
                                            onChange={(e) => updateNestedConfig('bittorrent', { encryption: e.target.value as EncryptionMode })}
                                            className="input"
                                        >
                                            <option value="Prefer">Prefer encryption (recommended)</option>
                                            <option value="ForceOn">Require encryption</option>
                                            <option value="ForceOff">Disable encryption</option>
                                        </select>
                                    </div>
                                </Section>

                                <Section title="Seeding Limits">
                                    <div className="space-y-4">
                                        <div className="grid grid-cols-2 gap-4">
                                            <div>
                                                <label className="text-sm text-aurora-dim mb-2 block">Share ratio limit</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    step="0.1"
                                                    value={config.bittorrent?.global_share_ratio_limit ?? ''}
                                                    onChange={(e) => updateNestedConfig('bittorrent', { 
                                                        global_share_ratio_limit: e.target.value ? parseFloat(e.target.value) : undefined 
                                                    })}
                                                    placeholder="No limit"
                                                    className="input"
                                                />
                                                <p className="text-xs text-aurora-muted mt-1">e.g., 2.0 = stop after uploading 2x the download size</p>
                                            </div>
                                            <div>
                                                <label className="text-sm text-aurora-dim mb-2 block">Seeding time limit (minutes)</label>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    value={config.bittorrent?.global_seeding_time_limit ?? ''}
                                                    onChange={(e) => updateNestedConfig('bittorrent', { 
                                                        global_seeding_time_limit: e.target.value ? parseInt(e.target.value) : undefined 
                                                    })}
                                                    placeholder="No limit"
                                                    className="input"
                                                />
                                            </div>
                                        </div>
                                        <div>
                                            <label className="text-sm text-aurora-dim mb-2 block">Action when limit reached</label>
                                            <select
                                                value={config.bittorrent?.share_limit_action || 'Nothing'}
                                                onChange={(e) => updateNestedConfig('bittorrent', { share_limit_action: e.target.value as ShareLimitAction })}
                                                className="input"
                                            >
                                                <option value="Nothing">Do nothing</option>
                                                <option value="Stop">Pause torrent</option>
                                                <option value="Remove">Remove torrent</option>
                                                <option value="RemoveWithContent">Remove torrent and files</option>
                                                <option value="EnableSuperSeeding">Enable super seeding</option>
                                            </select>
                                        </div>
                                    </div>
                                </Section>

                                <Section title="Additional Trackers">
                                    <div className="space-y-3">
                                        <Checkbox
                                            label="Automatically add trackers to new torrents"
                                            checked={config.bittorrent?.add_trackers_enabled || false}
                                            onChange={(v) => updateNestedConfig('bittorrent', { add_trackers_enabled: v })}
                                        />
                                        {config.bittorrent?.add_trackers_enabled && (
                                            <textarea
                                                value={(config.bittorrent?.additional_trackers || []).join('\n')}
                                                onChange={(e) => updateNestedConfig('bittorrent', { 
                                                    additional_trackers: e.target.value.split('\n').filter(t => t.trim()) 
                                                })}
                                                placeholder="One tracker URL per line"
                                                className="input min-h-[100px] font-mono text-sm"
                                            />
                                        )}
                                    </div>
                                </Section>
                            </div>
                        )}

                        {/* WebUI Tab */}
                        {activeTab === 'webui' && (
                            <div className="space-y-6">
                                <Section title="Web User Interface">
                                    <div className="space-y-4">
                                        <Checkbox
                                            label="Enable Web UI (Remote control from browsers)"
                                            checked={config.webui?.enabled || false}
                                            onChange={(v) => updateNestedConfig('webui', { enabled: v })}
                                        />
                                        {config.webui?.enabled && (
                                            <>
                                                <div className="grid grid-cols-2 gap-4">
                                                    <InputField
                                                        label="Listen Address"
                                                        value={config.webui?.address || '0.0.0.0'}
                                                        onChange={(v) => updateNestedConfig('webui', { address: v })}
                                                        placeholder="0.0.0.0"
                                                    />
                                                    <InputField
                                                        label="Port"
                                                        type="number"
                                                        value={config.webui?.port?.toString() || '8080'}
                                                        onChange={(v) => updateNestedConfig('webui', { port: parseInt(v) || 8080 })}
                                                    />
                                                </div>
                                                <div className="grid grid-cols-2 gap-4">
                                                    <InputField
                                                        label="Username"
                                                        value={config.webui?.username || 'admin'}
                                                        onChange={(v) => updateNestedConfig('webui', { username: v })}
                                                    />
                                                    <InputField
                                                        label="Password"
                                                        type="password"
                                                        value={config.webui?.password_hash || ''}
                                                        onChange={(v) => updateNestedConfig('webui', { password_hash: v })}
                                                        placeholder="Enter new password"
                                                    />
                                                </div>
                                                <Checkbox
                                                    label="Bypass authentication for localhost"
                                                    checked={config.webui?.localhost_auth_bypass || false}
                                                    onChange={(v) => updateNestedConfig('webui', { localhost_auth_bypass: v })}
                                                />
                                                <Checkbox
                                                    label="Enable UPnP for Web UI port"
                                                    checked={config.webui?.use_upnp || false}
                                                    onChange={(v) => updateNestedConfig('webui', { use_upnp: v })}
                                                />
                                            </>
                                        )}
                                    </div>
                                </Section>

                                {config.webui?.enabled && (
                                    <Section title="Security">
                                        <div className="space-y-3">
                                            <Checkbox
                                                label="Enable clickjacking protection"
                                                checked={config.webui?.clickjacking_protection !== false}
                                                onChange={(v) => updateNestedConfig('webui', { clickjacking_protection: v })}
                                            />
                                            <Checkbox
                                                label="Enable CSRF protection"
                                                checked={config.webui?.csrf_protection !== false}
                                                onChange={(v) => updateNestedConfig('webui', { csrf_protection: v })}
                                            />
                                            <Checkbox
                                                label="Enable Host header validation"
                                                checked={config.webui?.host_header_validation !== false}
                                                onChange={(v) => updateNestedConfig('webui', { host_header_validation: v })}
                                            />
                                            <Checkbox
                                                label="Enable HTTPS"
                                                checked={config.webui?.https_enabled || false}
                                                onChange={(v) => updateNestedConfig('webui', { https_enabled: v })}
                                            />
                                            {config.webui?.https_enabled && (
                                                <div className="grid grid-cols-2 gap-4 pl-6">
                                                    <InputField
                                                        label="Certificate path"
                                                        value={config.webui?.https_cert_path || ''}
                                                        onChange={(v) => updateNestedConfig('webui', { https_cert_path: v })}
                                                        placeholder="/path/to/cert.pem"
                                                    />
                                                    <InputField
                                                        label="Private key path"
                                                        value={config.webui?.https_key_path || ''}
                                                        onChange={(v) => updateNestedConfig('webui', { https_key_path: v })}
                                                        placeholder="/path/to/key.pem"
                                                    />
                                                </div>
                                            )}
                                        </div>
                                    </Section>
                                )}
                            </div>
                        )}

                        {/* Advanced Tab */}
                        {activeTab === 'advanced' && (
                            <div className="space-y-6">
                                <Section title="Behavior">
                                    <div className="space-y-3">
                                        <Checkbox
                                            label="Confirm before deleting torrents"
                                            checked={config.confirm_delete !== false}
                                            onChange={(v) => updateConfig('confirm_delete', v)}
                                        />
                                        <Checkbox
                                            label="Show desktop notifications"
                                            checked={config.show_notifications !== false}
                                            onChange={(v) => updateConfig('show_notifications', v)}
                                        />
                                        <Checkbox
                                            label="Start torrents when app launches"
                                            checked={config.start_on_launch !== false}
                                            onChange={(v) => updateConfig('start_on_launch', v)}
                                        />
                                    </div>
                                </Section>

                                <Section title="Power Management">
                                    <div className="space-y-3">
                                        <Checkbox
                                            label="Prevent system from sleeping while downloading"
                                            checked={config.prevent_sleep_downloading !== false}
                                            onChange={(v) => updateConfig('prevent_sleep_downloading', v)}
                                        />
                                        <Checkbox
                                            label="Prevent system from sleeping while seeding"
                                            checked={config.prevent_sleep_seeding || false}
                                            onChange={(v) => updateConfig('prevent_sleep_seeding', v)}
                                        />
                                    </div>
                                </Section>

                                <Section title="On Completion">
                                    <div>
                                        <label className="text-sm text-aurora-dim mb-2 block">When all downloads finish:</label>
                                        <select
                                            value={config.action_on_completion || 'none'}
                                            onChange={(e) => updateConfig('action_on_completion', e.target.value)}
                                            className="input"
                                        >
                                            <option value="none">Do nothing</option>
                                            <option value="exit">Exit application</option>
                                            <option value="shutdown">Shutdown computer</option>
                                            <option value="hibernate">Hibernate</option>
                                            <option value="sleep">Sleep</option>
                                        </select>
                                    </div>
                                </Section>

                                <Section title="UI">
                                    <div className="space-y-4">
                                        <div>
                                            <label className="text-sm text-aurora-dim mb-2 block">Theme</label>
                                            <select
                                                value={config.theme || 'dark'}
                                                onChange={(e) => updateConfig('theme', e.target.value)}
                                                className="input"
                                            >
                                                <option value="dark">Dark</option>
                                                <option value="light">Light</option>
                                                <option value="system">System</option>
                                            </select>
                                        </div>
                                        <Checkbox
                                            label="Show speed in title bar"
                                            checked={config.speed_in_title !== false}
                                            onChange={(v) => updateConfig('speed_in_title', v)}
                                        />
                                        <Checkbox
                                            label="Minimize to system tray"
                                            checked={config.minimize_to_tray || false}
                                            onChange={(v) => updateConfig('minimize_to_tray', v)}
                                        />
                                        <Checkbox
                                            label="Close to system tray"
                                            checked={config.close_to_tray || false}
                                            onChange={(v) => updateConfig('close_to_tray', v)}
                                        />
                                    </div>
                                </Section>

                                <Section title="RSS">
                                    <div className="space-y-4">
                                        <InputField
                                            label="RSS refresh interval (minutes)"
                                            type="number"
                                            value={config.rss_refresh_interval?.toString() || '30'}
                                            onChange={(v) => updateConfig('rss_refresh_interval', parseInt(v) || 30)}
                                        />
                                        <InputField
                                            label="Max articles per feed"
                                            type="number"
                                            value={config.rss_max_articles_per_feed?.toString() || '50'}
                                            onChange={(v) => updateConfig('rss_max_articles_per_feed', parseInt(v) || 50)}
                                        />
                                        <Checkbox
                                            label="Enable automatic RSS downloading"
                                            checked={config.rss_auto_download_enabled !== false}
                                            onChange={(v) => updateConfig('rss_auto_download_enabled', v)}
                                        />
                                    </div>
                                </Section>
                            </div>
                        )}
                    </div>
                </div>

                {/* Footer */}
                <div className="flex justify-end gap-3 p-6 border-t border-aurora-border/50">
                    <button onClick={onClose} className="btn-secondary">
                        Cancel
                    </button>
                    <button
                        onClick={handleSave}
                        disabled={saving || !hasChanges}
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

// =============================================================================
// Helper Components
// =============================================================================

function Section({ title, children }: { title: string; children: React.ReactNode }) {
    return (
        <div className="space-y-4">
            <h3 className="text-lg font-semibold text-aurora-text flex items-center gap-2">
                {title}
            </h3>
            <div className="pl-0">{children}</div>
        </div>
    );
}

function InputField({ 
    label, 
    value, 
    onChange, 
    type = 'text', 
    placeholder,
    icon
}: { 
    label: string; 
    value: string; 
    onChange: (v: string) => void; 
    type?: string;
    placeholder?: string;
    icon?: React.ReactNode;
}) {
    return (
        <div>
            <label className="flex items-center gap-2 text-sm font-medium text-aurora-dim mb-2">
                {icon}
                {label}
            </label>
            <input
                type={type}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
                className="input"
            />
        </div>
    );
}

function Checkbox({ 
    label, 
    checked, 
    onChange 
}: { 
    label: string; 
    checked: boolean; 
    onChange: (v: boolean) => void;
}) {
    return (
        <label className="flex items-center gap-3 cursor-pointer group" onClick={() => onChange(!checked)}>
            <div className={`w-5 h-5 rounded border flex items-center justify-center transition-colors ${
                checked 
                    ? 'bg-aurora-cyan border-aurora-cyan' 
                    : 'border-aurora-border group-hover:border-aurora-cyan/50'
            }`}>
                {checked && <Check size={14} className="text-aurora-void" />}
            </div>
            <span className="text-sm text-aurora-text">{label}</span>
        </label>
    );
}
