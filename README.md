# Run development environment

## Run the compiler in watch mode

```bash
cargo install watch
```

```bash
cargo watch -x "wasm-pack build --dev --target web"
```

## Run the server

```bash
npx server .
```

# For building the project

```bash
wasm-pack build --target web
```
