#!/usr/bin/env bash
# Assemble the static site (root page + all four wasm builds + songs) into dist/,
# keeping the same relative paths the page uses when served from the repo root.
# Build the wasm first: see README and the .github/workflows/pages.yml workflow.
set -euo pipefail
cd "$(dirname "$0")/.."

out=dist
rm -rf "$out"

files=(
    index.html index.js favicon.ico song.nbsx
    target/wasm32-unknown-emscripten/release/nbs-player-rs.js
    target/wasm32-unknown-emscripten/release/nbs_player_rs.wasm
    target/wasm32-unknown-unknown/release/nbs-player-macroquad.wasm
    crates/nbs-player-macroquad/web/mq_js_bundle.js
    crates/nbs-player-macroquad/web/nbs_audio.js
    crates/nbs-player-wgpu/web/pkg/nbs_player_wgpu.js
    crates/nbs-player-wgpu/web/pkg/nbs_player_wgpu_bg.wasm
    crates/nbs-player-canvas/web/pkg/nbs_player_canvas.js
    crates/nbs-player-canvas/web/pkg/nbs_player_canvas_bg.wasm
)
for f in "${files[@]}"; do
    [ -f "$f" ] || { echo "missing $f (build it first)" >&2; exit 1; }
    mkdir -p "$out/$(dirname "$f")"
    cp "$f" "$out/$f"
done
cp -r test-assets "$out/test-assets"
touch "$out/.nojekyll"
echo "site in $out/ ($(du -sh "$out" | cut -f1))"
