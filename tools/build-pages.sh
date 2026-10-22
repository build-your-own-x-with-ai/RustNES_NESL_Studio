#!/usr/bin/env bash
# Builds the static NESL Studio site into docs/, ready for GitHub Pages.
#
# Requires: rust (with the wasm32-unknown-unknown target), wasm-pack, node.
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-pack   (or grab a release binary)
#
# Then: Settings > Pages > Source: "Deploy from a branch" > branch main, /docs

set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> wasm-pack build (release)"
wasm-pack build crates/nesc-wasm --release --target web --out-name nesc_wasm

echo "==> rendering docs/"
node tools/render-pages.mjs

echo
echo "docs/ is ready. To preview:  python3 -m http.server 8000 --directory docs"
