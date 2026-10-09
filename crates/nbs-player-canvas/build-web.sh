#!/usr/bin/env bash
# Build the canvas player for the browser into web/pkg and (optionally) serve it.
#   ./build-web.sh          build only
#   ./build-web.sh serve    build, then serve on http://localhost:8080
# Needs a wasm-bindgen CLI matching the wasm-bindgen crate version in Cargo.lock:
#   cargo install wasm-bindgen-cli --version <that version> --root target/tools
set -euo pipefail
cd "$(dirname "$0")"
root=../..

want=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/[version = "]/,""); print; exit}' "$root/Cargo.lock")
cli="$root/target/tools/bin/wasm-bindgen"
[ -x "$cli" ] || cli=$(command -v wasm-bindgen || true)
if [ -z "$cli" ] || [ "$("$cli" --version | awk '{print $2}')" != "$want" ]; then
    echo "need wasm-bindgen CLI $want:" >&2
    echo "  cargo install wasm-bindgen-cli --version $want --root target/tools" >&2
    exit 1
fi

# the `web` profile (workspace Cargo.toml) optimizes for size: smaller than the GPU builds
cargo build --profile web --target wasm32-unknown-unknown -p nbs-player-canvas --lib
"$cli" --target web --no-typescript --remove-name-section --remove-producers-section \
    --out-dir web/pkg "$root/target/wasm32-unknown-unknown/web/nbs_player_canvas.wasm"

if [ "${1:-}" = "serve" ]; then
    cd web && python3 -m http.server 8080
fi
