# Run development environment

## Compile and Running for WebAssembly

you need to have the Emscripten SDK installed. You can find instructions on how to install it [here](https://emscripten.org/docs/getting_started/downloads.html).
After installing the SDK, you need to activate it. You can do this by running the following command in your terminal:

```bash
EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY" cargo build --release --target wasm32-unknown-emscripten
```

```bash
npx serve .
```

# Running locally

```bash
cargo run -- "$(cat <<EOF
{
  "song_url": "./test-assets/turkish_march.nbs",
  "font_id": 5,
  "window_width": 1280,
  "window_height": 720,
  "theme": {
    "background_color": "#2E1A1A",
    "accent_color": "#FF0000",
    "text_color": "#E0E0E0",
    "white_key_color": "#444444",
    "black_key_color": "#1A1A1A",
    "white_text_key_color": "#E0E0E0",
    "black_text_key_color": "#FF0000"
  }
}
EOF
)"
```

You can change the arguments as you like.
