#!/bin/sh
# Local development: the Leptos server with live reload, plus the CSS watcher.
set -eu

export JWT_SECRET="${JWT_SECRET:-dev_secret}"
export RUST_LOG="${RUST_LOG:-info}"

# cargo-leptos reads the built CSS (its `style-file`), so it has to exist first
npm run build:styles

npm run watch:styles &
css=$!
cargo leptos watch &
leptos=$!
trap 'kill $css $leptos 2>/dev/null' EXIT INT TERM

wait $leptos
