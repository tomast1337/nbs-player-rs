# Run development environment

## Run the compiler in watch mode

```bash
cargo install watchexec
```

```bash
watchexec -e rs "wasm-pack build --dev --target web"
```

## Run the server

```bash
npx serve .
```

# For building the project

```bash
wasm-pack build --target web
```
