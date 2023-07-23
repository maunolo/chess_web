/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
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
        },
        animation: {
            'notify-show': 'notify-show 5s ease-in-out',
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
        minHeight: {
            '15': '3.75rem',
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
            '51.25': '12.8125rem',
            '54.5': '13.625rem',
            '58': '14.5rem',
            '58.5': '14.625rem',
            '62': '15.5rem',
            '62.5': '15.625rem',
            '68': '17rem',
            '69': '17.25rem',
            '73': '18.25rem',
            '74': '18.5rem',
            '88': '22rem',
        }
    },
  },
  plugins: [],
}
