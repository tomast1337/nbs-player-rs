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
    "background_color": "#4B0082",
    "accent_color": "#FFD700",
    "text_color": "#E6E6FA",
    "white_key_color": "#F0E68C",
    "black_key_color": "#483D8B",
    "white_text_key_color": "#1A1A1A",
    "black_text_key_color": "#E6E6FA"
  }
}
EOF
)"
```

You can change the arguments as you like.
