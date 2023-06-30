/** @type {import('tailwindcss').Config} */
module.exports = {
  content: { 
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
        spacing: {
            '2.75': '0.688rem',
            '15': '3.75rem',
            '45': '11.25rem',
            '51': '12.75rem',
        }
    },
  },
  plugins: [],
}