module.exports = {
  plugins: {
    'postcss-import': {},
    'postcss-at-rules-variables': {},
    'postcss-each': {},
    'postcss-for': {},
    'tailwindcss/nesting': {},
    tailwindcss: {},
    ...(process.env.NODE_ENV === 'production' ? { cssnano: {} } : {})
  }
}