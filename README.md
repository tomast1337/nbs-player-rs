# Note Block Studio Rust Player

![image](https://github.com/user-attachments/assets/6db2b940-c75f-408c-8398-eeead55f452f)

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

## Compile and Running for WebAssembly

you need to have the Emscripten SDK installed. You can find instructions on how to install it [here](https://emscripten.org/docs/getting_started/downloads.html).
After installing the SDK, you need to activate it. You can do this by running the following command in your terminal:

On Linux or MacOS:

```bash
EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY" cargo build --release --target wasm32-unknown-emscripten
```

On Windows:

```bash
set EMCC_CFLAGS=-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY
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

## Compile and Running for Native

```bash
cargo run -- "$(cat <<EOF
{
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

The program will always look for a file called `song.nbsx` in the current working directory, so you need to place your song there.

You can change the arguments as you like.

# License

This project is licensed under the GNU Affero General Public License v3.0. See the [LICENSE](LICENSE) file for details.
The assets are licensed under different licenses, please check the `test-assets` folder for more information.
