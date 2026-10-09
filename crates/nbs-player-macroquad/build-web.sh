#!/usr/bin/env bash
# Build the wasm player into web/ and (optionally) serve it.
#   ./build-web.sh          build only
#   ./build-web.sh serve    build, then serve on http://localhost:8080
# Put a `song.nbsx` (or any .nbs/.zip renamed) next to index.html to play your own song;
# without one the bundled demo plays.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown -p nbs-player-macroquad
cp ../../target/wasm32-unknown-unknown/release/nbs-player-macroquad.wasm web/
if [ "${1:-}" = "serve" ]; then
    cd web && python3 -m http.server 8080
fi
