import { motion } from 'framer-motion';
import { CheckCircle, XCircle, Info, X } from 'lucide-react';

interface ToastProps {
    id: string;
    type: 'success' | 'error' | 'info';
    message: string;
    onClose: () => void;
}

export default function Toast({ type, message, onClose }: ToastProps) {
    const icons = {
        success: CheckCircle,
        error: XCircle,
        info: Info,
    };

    const colors = {
        success: 'border-aurora-teal bg-aurora-teal/10 text-aurora-teal',
        error: 'border-aurora-rose bg-aurora-rose/10 text-aurora-rose',
        info: 'border-aurora-cyan bg-aurora-cyan/10 text-aurora-cyan',
    };

    const Icon = icons[type];

    return (
        <motion.div
            initial={{ opacity: 0, x: 100, scale: 0.9 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 100, scale: 0.9 }}
            className={`flex items-center gap-3 px-4 py-3 rounded-xl border backdrop-blur-sm shadow-lg ${colors[type]}`}
        >
            <Icon className="w-5 h-5 flex-shrink-0" />
            <p className="text-sm font-medium text-aurora-text flex-1">{message}</p>
            <button
                onClick={onClose}
                className="p-1 hover:bg-white/10 rounded transition-colors"
            >
                <X className="w-4 h-4" />
            </button>
        </motion.div>
    );
}

