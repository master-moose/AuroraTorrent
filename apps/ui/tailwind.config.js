/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                aurora: {
                    // Deep space background
                    void: '#0a0a0f',
                    deep: '#0f0f1a',
                    space: '#141428',
                    night: '#1a1a2e',
                    
                    // Aurora accent colors
                    cyan: '#00d4ff',
                    teal: '#00ffc8',
                    violet: '#8b5cf6',
                    magenta: '#d946ef',
                    rose: '#fb7185',
                    
                    // Subtle greys
                    muted: '#6b7280',
                    subtle: '#374151',
                    border: '#1f2937',
                    
                    // Text
                    text: '#f3f4f6',
                    dim: '#9ca3af',
                },
            },
            fontFamily: {
                sans: ['Outfit', 'system-ui', 'sans-serif'],
                mono: ['JetBrains Mono', 'monospace'],
            },
            backgroundImage: {
                'aurora-gradient': 'linear-gradient(135deg, rgba(0, 212, 255, 0.1) 0%, rgba(139, 92, 246, 0.1) 50%, rgba(217, 70, 239, 0.05) 100%)',
                'aurora-glow': 'radial-gradient(ellipse at top, rgba(0, 212, 255, 0.15) 0%, transparent 50%)',
                'card-gradient': 'linear-gradient(180deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0) 100%)',
            },
            animation: {
                'pulse-slow': 'pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                'shimmer': 'shimmer 2s linear infinite',
                'float': 'float 6s ease-in-out infinite',
            },
            keyframes: {
                shimmer: {
                    '0%': { backgroundPosition: '-200% 0' },
                    '100%': { backgroundPosition: '200% 0' },
                },
                float: {
                    '0%, 100%': { transform: 'translateY(0)' },
                    '50%': { transform: 'translateY(-10px)' },
                },
            },
            boxShadow: {
                'aurora': '0 0 40px rgba(0, 212, 255, 0.15)',
                'aurora-strong': '0 0 60px rgba(0, 212, 255, 0.25)',
                'card': '0 4px 20px rgba(0, 0, 0, 0.3)',
            },
        },
    },
    plugins: [],
}
