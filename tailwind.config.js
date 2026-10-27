/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
        // The Slate palette: a near-black ground, two raised surfaces, and one accent
        colors: {
            ink: {
                950: '#0f1114',
                900: '#15181c',
                800: '#1f242a',
                700: '#2a3139',
                600: '#3a434d',
                400: '#5c6672',
                300: '#9aa3ad',
                100: '#e8ebef',
            },
            accent: {
                DEFAULT: '#f2b84b',
                hover: '#f6c86f',
            },
            coral: '#f0857d',
            mint: '#5ab07a',
        },
        fontFamily: {
            sans: ['Inter', 'system-ui', 'sans-serif'],
            mono: ['"Fira Code"', 'ui-monospace', 'monospace'],
        },
        backgroundPosition: {
            'piece': 'center bottom .1rem',
        },
        backgroundSize: {
            '90': '90%',
        },
        keyframes: {
            'notify-show': {
                '0%': { transform: 'translateY(-100%)' },
                '10%': { transform: 'translateY(0)' },
                '90%': { transform: 'translateY(0)' },
                '100%': { transform: 'translateY(-100%)' },
            },
            'pulse-brightness': {
                '0%, 100%': { filter: 'brightness(0.9)' },
                '50%': { filter: 'brightness(1)' },
            },
            // the avatar of the side to move: a steady ring and a fading halo
            'turn-pulse': {
                '0%': { boxShadow: '0 0 0 2px #f2b84b, 0 0 0 2px rgba(242, 184, 75, 0.6)' },
                '70%': { boxShadow: '0 0 0 2px #f2b84b, 0 0 0 12px rgba(242, 184, 75, 0)' },
                '100%': { boxShadow: '0 0 0 2px #f2b84b, 0 0 0 12px rgba(242, 184, 75, 0)' },
            },
        },
        animation: {
            'notify-show': 'notify-show 5s ease-in-out',
            'pulse-brightness': 'pulse-brightness 2s ease-in-out infinite',
            'turn-pulse': 'turn-pulse 1.8s ease-out infinite',
        },
        spacing: {
            '15': '3.75rem',
        },
    },
  },
  plugins: [],
}
