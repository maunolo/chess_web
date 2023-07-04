/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
        keyframes: {
            'pulse-brightness': {
                '0%, 100%': { filter: 'brightness(0.9)' },
                '50%': { filter: 'brightness(1)' },
            },
        },
        animation: {
            'pulse-brightness': 'pulse-brightness 2s ease-in-out infinite',
        },
        maxWidth: {
            '32': '8rem',
            '37': '9.25rem',
            '40': '10rem',
            '41': '10.25rem',
            '44': '11rem',
            '48': '12rem',
        },
        maxHeight: {
            '90': '22.5rem',
        },
        spacing: {
            '2.75': '0.688rem',
            '7.5': '1.875rem',
            '12.5': '3.125rem',
            '15': '3.75rem',
            '23': '5.75rem',
            '25': '6.25rem',
            '30': '7.5rem',
            '31': '7.75rem',
            '37': '9.25rem',
            '41': '10.25rem',
            '45': '11.25rem',
            '49': '12.25rem',
            '51': '12.75rem',
            '88': '22rem',
        }
    },
  },
  plugins: [],
}