import type { Config } from 'tailwindcss'

export default {
  darkMode: 'class',
  content: [
    './app/**/*.{vue,js,ts,jsx,tsx}',
    './components/**/*.{vue,js,ts,jsx,tsx}',
    './pages/**/*.{vue,js,ts,jsx,tsx}',
    './*.vue'
  ],
  theme: {
    extend: {
      colors: {
        atena: {
          bg: '#090a0f',
          surface: '#11131b',
          elevated: '#171a26',
          hover: '#1f2436',
          active: '#272e45',
          border: '#23283b',
          'border-subtle': '#1c2030',
          'border-focus': '#6366f1',
          accent: '#6366f1',
          'accent-hover': '#4f46e5',
          'accent-glow': 'rgba(99, 102, 241, 0.25)',
          secondary: '#a855f7',
          muted: '#64748b',
          text: '#f1f5f9',
          'text-dim': '#94a3b8'
        }
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'monospace']
      },
      animation: {
        'pulse-subtle': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-left': 'slideLeft 0.25s cubic-bezier(0.16, 1, 0.3, 1)'
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'scale(0.98)' },
          '100%': { opacity: '1', transform: 'scale(1)' }
        },
        slideLeft: {
          '0%': { transform: 'translateX(100%)' },
          '100%': { transform: 'translateX(0)' }
        }
      }
    },
  },
  plugins: [],
} satisfies Config
