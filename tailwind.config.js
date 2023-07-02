/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
        maxWidth: {
            '32': '8rem',
            '40': '10rem',
        },
        spacing: {
            '2.75': '0.688rem',
            '7.5': '1.875rem',
            '15': '3.75rem',
            '23': '5.75rem',
            '30': '7.5rem',
            '45': '11.25rem',
            '51': '12.75rem',
        }
    },
  },
  plugins: [],
}