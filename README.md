# Note Block Studio Rust Player

![image](https://github.com/user-attachments/assets/bc7b8387-dcbe-4f2a-b93e-fef2b627dd43)

This is a Rust player for the Note Block Studio format. It is a work in progress and is not yet complete.

A web version is available at <https://tomast1337.github.io/nbs-player-rs>, it also demostrate a way to make playlists commanded by the browser.

## Features

- Play Note Block Studio songs
- Support for custom instruments
- Support for custom themes and fonts
- Play songs in a web browser using WebAssembly
- Support for custom key bindings

## Requirements

- Rust 1.85.0 or later
- Emscripten SDK (for WebAssembly)

# Run development environment

Serve the web page (all frontends) with `bun scripts/serve.ts [port]`; build the wasm first (below, plus `crates/nbs-player-macroquad/build-web.sh`, `crates/nbs-player-wgpu/build-web.sh` and `crates/nbs-player-canvas/build-web.sh`).

## Compile and Running for WebAssembly

you need to have the Emscripten SDK installed. You can find instructions on how to install it [here](https://emscripten.org/docs/getting_started/downloads.html).
After installing the SDK, you need to activate it. You can do this by running the following command in your terminal:

On Linux or MacOS:

```bash
EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sJSPI" cargo build --release --target wasm32-unknown-emscripten
```

On Windows:

```bash
set EMCC_CFLAGS=-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sJSPI
cargo build --release --target wasm32-unknown-emscripten
```

After building the project, you can run the following command to start a local server and serve the files:

With node.js:

```bash
npx serve .
```

With python:

```bash
python3 -m http.server
```

Then, open your browser the link given by the server.

The build uses JSPI rather than Asyncify: rustc links with `-fwasm-exceptions`, which binaryen's Asyncify pass cannot instrument (it aborts in `Asyncify.cpp`). JSPI needs a browser that ships it (Chrome/Edge 137+; check current Firefox and Safari support).

## Compile and Running for Native

```bash
cargo run -- "$(cat <<EOF
{
  "font_id": "PixelPlay",
  "background": "Plasma",
  "window_width": 1280,
  "window_height": 720,
  "theme": {
    "background_color": "#FFE4E1",
    "accent_color": "#D4A59A",
    "text_color": "#5D4037",
    "white_key_color": "#FFF0F5",
    "black_key_color": "#E6C7C2",
    "white_text_key_color": "#5D4037",
    "black_text_key_color": "#D4A59A"
  }
}
EOF
)"
```

The program will always look for a file called `song.nbsx` in the current working directory, so you need to place your song there.

You can change the arguments as you like.

## Project layout

Cargo workspace; the player is split so rendering and audio can be swapped independently.

- `crates/nbs-player-core`: everything that is not a window, a GPU or an audio device. It holds the whole player (`app::App`): song loading, playback (`Player`), piano/note/controls layout, an immediate-mode UI (`ui`), the profiler and its overlay, and the audio side (`AudioBackend` trait, Ogg decoding, a software `Mixer`). A frame is `app.frame(input, dt, size, &mut renderer, &mut audio)`. Compiles for wasm and is unit-tested headless (`cargo test -p nbs-player-core`).
- `src/` (the raylib frontend): implements the core's `Renderer` (`src/render.rs`, plus fonts, textures and GLSL shaders), reads `InputState` from raylib, and provides audio backends (`src/audio/`).

- `crates/nbs-player-macroquad`: second frontend. Same core, drawn with macroquad; builds for native and `wasm32-unknown-unknown`. `cargo run -p nbs-player-macroquad -- [json-config] [song-path]`. Web: `crates/nbs-player-macroquad/build-web.sh serve` builds the wasm into `web/` and serves it on :8080 (drop a `song.nbsx` next to `index.html` to play your own song, edit `config.json` for the theme). Web audio is the core mixer pulled by a small Web Audio plugin (`web/nbs_audio.js`), not cpal: cpal's wasm host needs wasm-bindgen, which does not coexist with macroquad's loader.
- `crates/nbs-player-wgpu`: third frontend, wgpu + winit (Vulkan/Metal/DX12). `cargo run -p nbs-player-wgpu -- [json-config] [song-path]`. Rects, sprites and glyphs (fontdue atlas) are batched into one textured-quad pipeline; the background shaders are the shared GLSL files converted to WGSL by naga (`cargo run -p nbs-player-wgpu --example convert_shaders`; output committed in `assets/shaders/wgsl/`, a test fails when it goes stale). Audio is cpal. Web: `crates/nbs-player-wgpu/build-web.sh [serve]` builds a wasm-bindgen module into `web/pkg` (needs a `wasm-bindgen` CLI equal to the crate version in `Cargo.lock`; the script prints the install command). WebGPU is used when the browser has it, WebGL2 otherwise. Audio starts on the first click or key, as browsers require.
- `crates/nbs-player-canvas`: fourth frontend, a 2D `<canvas>` (web only, no GPU API, so it runs where WebGPU/WebGL do not). Sprites are tinted through cached offscreen canvases and the font loads through `FontFace`. The animated backgrounds are a CPU plasma in the theme colors, not the shaders. `crates/nbs-player-canvas/build-web.sh [serve]`; audio is cpal, started on the first click or key.
- `crates/nbs-player-cpal`: cpal output for the core mixer, shared by frontends.

To add another frontend (wgpu, macroquad, canvas...), implement `render::Renderer` (7 draw calls and text measuring), fill `render::InputState` each frame, and pick an `AudioBackend`.

### Audio backends

Pick one with `"audio_backend"` in the JSON config:

- `"Raylib"` (default): one raylib `Sound` per note, as before.
- `"Cpal"`: the core software mixer behind a cpal output stream. Notes are just voice-start events, so triggering a tick is ~100x cheaper than the raylib path. Not available on the emscripten target (cpal has no emscripten host); it falls back to raylib there.

Render a song to a WAV without any device: `cargo run --release -p nbs-player-core --example render_wav -- song.nbs out.wav [seconds]`.

## Profiler

Built-in scoped stack profiler (`crates/nbs-player-core/src/profiler.rs`, overlay in `debug_overlay.rs`). In-app: **F3** toggles the overlay (avg / peak ms, call count per nested scope), **F4** writes `profile.folded`. Render a flamegraph with `cargo install inferno` then `inferno-flamegraph < profile.folded > flame.svg`. Add scopes with `let _p = profiler::scope("name");`. Headless: `NBS_PROFILE_FRAMES=900 cargo run --release -- '<json>'` profiles that many frames, logs the report and exits.

# License

This project is licensed under the GNU Affero General Public License v3.0. See the [LICENSE](LICENSE) file for details.
The assets are licensed under different licenses, please check the `test-assets` folder for more information.
