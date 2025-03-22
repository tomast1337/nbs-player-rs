# Run development environment

## Compile for WebAssembly

you need to have the Emscripten SDK installed. You can find instructions on how to install it [here](https://emscripten.org/docs/getting_started/downloads.html).
After installing the SDK, you need to activate it. You can do this by running the following command in your terminal:

```bash
EMCC_CFLAGS="-sUSE_GLFW=3 -sGL_ENABLE_GET_PROC_ADDRESS -sASYNCIFY" cargo build --release --target wasm32-unknown-emscripten
```

```bash
npx serve .
```
