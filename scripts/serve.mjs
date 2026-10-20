// A tiny static file server for the example page and the e2e tests. It sends the headers that
// docs/integration.md asks for: application/wasm, immutable caching for hashed files, no-cache
// for the viewer page. No COOP/COEP: the SDK does not need cross-origin isolation.
//
// Usage: node scripts/serve.mjs [--port 8780] [--root .]
// The SDK is served from /sdk/ (the newest dist/free-pdf-sdk-*/ folder) and the example from /.

import { createServer } from "node:http";
import { readFile, readdir, stat } from "node:fs/promises";
import { extname, join, normalize, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const repo = resolve(here, "..");

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
}

const port = Number(arg("port", process.env.PORT ?? "8780"));
const exampleRoot = resolve(repo, arg("root", "examples/vanilla"));

async function sdkRoot() {
  const explicit = arg("sdk", process.env.FPS_SDK_DIR ?? "");
  if (explicit) return resolve(repo, explicit);
  const dist = join(repo, "dist");
  const entries = (await readdir(dist, { withFileTypes: true })).filter((e) => e.isDirectory() && e.name.startsWith("free-pdf-sdk-"));
  entries.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }));
  const last = entries.at(-1);
  if (!last) throw new Error("no dist/free-pdf-sdk-*/ folder: run scripts/build.ps1 first");
  return join(dist, last.name);
}

const types = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".svg": "image/svg+xml",
  ".pdf": "application/pdf",
  ".xfdf": "application/vnd.adobe.xfdf",
  ".txt": "text/plain; charset=utf-8",
  ".md": "text/markdown; charset=utf-8",
  ".ts": "text/plain; charset=utf-8",
};

const hashed = /-[0-9a-f]{10}(_bg)?\.(js|wasm)$/;

function inside(root, path) {
  const full = normalize(join(root, path));
  return full === root || full.startsWith(root + sep) ? full : null;
}

const sdk = await sdkRoot();

createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? "/", "http://localhost");
    let path = decodeURIComponent(url.pathname);
    let root = exampleRoot;
    if (path.startsWith("/sdk/")) {
      root = sdk;
      path = path.slice("/sdk".length);
    } else if (path.startsWith("/fixtures/")) {
      root = join(repo, "tests", "fixtures");
      path = path.slice("/fixtures".length);
    }
    if (path.endsWith("/")) path += "index.html";
    const file = inside(root, path);
    if (!file) {
      res.writeHead(403).end();
      return;
    }
    const info = await stat(file).catch(() => null);
    if (!info?.isFile()) {
      res.writeHead(404, { "content-type": "text/plain" }).end("not found");
      return;
    }
    const body = await readFile(file);
    const cache = hashed.test(file) ? "public, max-age=31536000, immutable" : "no-cache";
    res.writeHead(200, {
      "content-type": types[extname(file)] ?? "application/octet-stream",
      "cache-control": cache,
      "x-content-type-options": "nosniff",
    });
    res.end(body);
  } catch (e) {
    res.writeHead(500, { "content-type": "text/plain" }).end(String(e));
  }
}).listen(port, "127.0.0.1", () => {
  console.log(`Free PDF SDK example on http://127.0.0.1:${port}/ (SDK from ${sdk})`);
});
