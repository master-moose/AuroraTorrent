import { Home, Library, Search, Settings, Download, Zap } from 'lucide-react';
import { motion } from 'framer-motion';

interface SidebarProps {
    currentView: string;
    setView: (v: 'home' | 'library' | 'search') => void;
    activeDownloads: number;
    onOpenSettings: () => void;
}

export default function Sidebar({ currentView, setView, activeDownloads, onOpenSettings }: SidebarProps) {
    const navItems = [
        { id: 'home', icon: Home, label: 'Home' },
        { id: 'library', icon: Library, label: 'Library' },
        { id: 'search', icon: Search, label: 'Search' },
    ] as const;

    return (
        <div className="w-64 flex flex-col p-4 border-r border-aurora-border/30">
            {/* Logo */}
            <div className="flex items-center gap-3 px-3 py-4 mb-6">
                <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-aurora-cyan to-aurora-violet flex items-center justify-center shadow-aurora">
                    <Zap className="w-5 h-5 text-white" />
                </div>
                <div>
                    <h1 className="font-bold text-lg text-aurora-text">Aurora</h1>
                    <p className="text-xs text-aurora-dim">Torrent Client</p>
                </div>
            </div>

            {/* Navigation */}
            <nav className="flex-1 space-y-1">
                {navItems.map(({ id, icon: Icon, label }) => (
                    <motion.button
                        key={id}
                        onClick={() => setView(id)}
                        className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 ${
                            currentView === id 
                                ? 'bg-aurora-cyan/10 text-aurora-cyan' 
                                : 'text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50'
                        }`}
                        whileHover={{ x: 4 }}
                        whileTap={{ scale: 0.98 }}
                    >
                        <Icon size={20} />
                        <span className="font-medium">{label}</span>
                        {id === 'library' && currentView === id && (
                            <motion.div 
                                layoutId="nav-indicator"
                                className="ml-auto w-1.5 h-1.5 rounded-full bg-aurora-cyan"
                            />
                        )}
                    </motion.button>
                ))}
            </nav>

            {/* Active Downloads Card */}
            {activeDownloads > 0 && (
                <motion.div 
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="card p-4 mb-4"
                >
                    <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-lg bg-aurora-cyan/10 flex items-center justify-center">
                            <Download className="w-5 h-5 text-aurora-cyan animate-pulse" />
                        </div>
                        <div className="flex-1 min-w-0">
                            <p className="font-medium text-sm text-aurora-text">Active Downloads</p>
                            <p className="text-xs text-aurora-dim">{activeDownloads} in progress</p>
                        </div>
                    </div>
                </motion.div>
            )}

            {/* Settings Button */}
            <button
                onClick={onOpenSettings}
                className="flex items-center gap-3 px-4 py-3 rounded-xl text-aurora-dim hover:text-aurora-text hover:bg-aurora-night/50 transition-all duration-200"
            >
                <Settings size={20} />
                <span className="font-medium">Settings</span>
            </button>

            {/* Version */}
            <div className="px-4 py-2 mt-2">
                <p className="text-xs text-aurora-muted">v0.1.0</p>
            </div>
        </div>
    );
}
