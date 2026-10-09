#!/usr/bin/env bash
# Build the raylib (emscripten) web player into target/wasm32-unknown-emscripten/release.
# Needs the Emscripten SDK on PATH (emcc). JSPI instead of Asyncify: see README.
set -euo pipefail
cd "$(dirname "$0")/.."
command -v emcc >/dev/null || { echo "emcc not found: install and activate the Emscripten SDK" >&2; exit 1; }
EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sJSPI" \
    cargo build --release --target wasm32-unknown-emscripten
