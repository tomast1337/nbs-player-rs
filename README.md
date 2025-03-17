# Run development environment

## Run the compiler in watch mode

```bash
cargo install watchexec
```

```bash
watchexec -e rs "cargo build --target wasm32-unknown-unknown --release"
```

## Run the server

```bash
npx serve .
```
