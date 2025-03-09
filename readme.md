cargo watch -x "wasm-pack build --target web"

wasm-pack build --dev --target web

# Run the server

npx server .
