// Static dev server for the web page and its build outputs.
//   bun scripts/serve.ts [port]        (default 8080, or $PORT)
// Serves the repo root: index.html, index.js, test-assets/, the emscripten build in
// target/wasm32-unknown-emscripten and the macroquad build in
// target/wasm32-unknown-unknown + crates/nbs-player-macroquad/web.
import { existsSync, statSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const port = Number(process.argv[2] ?? process.env.PORT ?? 8080);

const types: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".ico": "image/x-icon",
  ".png": "image/png",
  ".css": "text/css; charset=utf-8",
};

// Files the page needs that only exist after a build; warn once at startup.
const expected = [
  "target/wasm32-unknown-emscripten/release/nbs-player-rs.js",
  "target/wasm32-unknown-unknown/release/nbs-player-macroquad.wasm",
  "crates/nbs-player-wgpu/web/pkg/nbs_player_wgpu.js",
  "crates/nbs-player-canvas/web/pkg/nbs_player_canvas.js",
];
for (const f of expected) {
  if (!existsSync(join(root, f))) {
    console.warn(`missing ${f}`);
  }
}

const server = Bun.serve({
  port,
  fetch(req) {
    const url = new URL(req.url);
    let path = decodeURIComponent(url.pathname);
    if (path.endsWith("/")) path += "index.html";

    // Keep requests inside the repo root.
    const file = normalize(join(root, path));
    if (!file.startsWith(root + "/") && file !== root) {
      return new Response("forbidden", { status: 403 });
    }
    if (!existsSync(file) || !statSync(file).isFile()) {
      return new Response("not found", { status: 404 });
    }

    return new Response(Bun.file(file), {
      headers: {
        "Content-Type": types[extname(file)] ?? "application/octet-stream",
        // Always pick up fresh builds.
        "Cache-Control": "no-store",
      },
    });
  },
});

console.log(`serving ${root}`);
console.log(`  raylib:    http://localhost:${server.port}/?renderer=raylib`);
console.log(`  macroquad: http://localhost:${server.port}/?renderer=macroquad`);
console.log(`  wgpu:      http://localhost:${server.port}/?renderer=wgpu`);
console.log(`  canvas:    http://localhost:${server.port}/?renderer=canvas`);
