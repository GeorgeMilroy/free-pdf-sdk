# Integration and hosting

## What you deploy

`scripts/build.ps1` produces `dist/free-pdf-sdk-<version>/`:

```
free-pdf-sdk-0.1.0/
  free-pdf-sdk.js          ESM wrapper (import this)
  free-pdf-sdk.d.ts        TypeScript types
  free-pdf-sdk.js.map
  version.json             { "sdk": "0.1.0", "pdfcraftRev": "<40 hex>" }
  viewer/
    index.html             the iframe page (not hashed)
    fps-viewer-<hash>.js   wasm-bindgen glue
    fps-viewer-<hash>_bg.wasm
  LICENSE-MIT  LICENSE-APACHE  NOTICE  README.md  CHANGELOG.md
  third-party/             licences and notices of everything the wasm embeds
```

Copy the whole folder into your site, keeping the version in the path, for example
`public/free-pdf-sdk/0.1.0/`. Point `baseUrl` at it.

## Requirements

1. **Same origin.** The SDK folder must be served from the same origin (scheme, host and port)
   as the page that calls `FreePdf.init`. The wrapper refuses another origin, and the viewer only
   talks to a parent on its own origin. The PDF itself may come from anywhere the page can
   `fetch` (for example a Firebase Storage download URL with CORS configured).
2. **`Content-Type: application/wasm`** for `.wasm` files. Without it the browser falls back to a
   slower load (and some servers send `application/octet-stream`, which fails streaming
   compilation).
3. **WebGL.** The viewer draws with WebGL (egui's Glow renderer). Every current browser has it;
   locked-down machines without GPU acceleration fall back to software rendering.
4. **No COOP/COEP needed.** The viewer uses no `SharedArrayBuffer` and no threads, so it does
   not need cross-origin isolation (`Cross-Origin-Opener-Policy`/`Cross-Origin-Embedder-Policy`).
   Don't add them for the SDK: COEP would also block cross-origin PDFs and Firebase Auth popups.
5. **Framing.** The page frames the viewer on its own origin: don't send `X-Frame-Options: DENY`
   for the SDK folder (`SAMEORIGIN` is fine), and if you use CSP `frame-ancestors`, allow `'self'`.
   With a CSP, the viewer needs `script-src 'self' 'wasm-unsafe-eval'` and `worker-src` is not
   needed. If the host page has a CSP, it needs `frame-src 'self'`.

## Caching

| Files | Cache-Control |
|---|---|
| `viewer/fps-viewer-<hash>.js`, `viewer/fps-viewer-<hash>_bg.wasm` | `public, max-age=31536000, immutable` (content-hashed) |
| everything else under a versioned folder (`/free-pdf-sdk/0.1.0/…`) | `public, max-age=31536000, immutable` is safe, because a new version is a new folder |
| `viewer/index.html` | `no-cache` (it names the hashed files; keep it fresh in case a folder is re-deployed) |

Serve `.wasm` compressed (brotli or gzip) if your host can; see the size in the release notes
(about 9 MB brotli for 0.1.0, 37 MB uncompressed).

## Firebase Hosting example

With the SDK copied to `public/free-pdf-sdk/0.1.0/` (Vue CLI copies `public/` into `dist/`):

```json
{
  "hosting": {
    "public": "dist",
    "headers": [
      {
        "source": "/free-pdf-sdk/**",
        "headers": [{ "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }]
      },
      {
        "source": "/free-pdf-sdk/**/viewer/index.html",
        "headers": [{ "key": "Cache-Control", "value": "no-cache" }]
      },
      {
        "source": "**/*.wasm",
        "headers": [{ "key": "Content-Type", "value": "application/wasm" }]
      }
    ],
    "rewrites": [
      { "source": "/free-pdf-sdk/**", "destination": "/free-pdf-sdk/404.html" },
      { "source": "**", "destination": "/index.html" }
    ]
  }
}
```

Notes:

* Firebase applies the *last* matching header rule for a key, so the `no-cache` rule for
  `viewer/index.html` must come after the folder rule.
* Firebase Hosting already serves `.wasm` as `application/wasm` and compresses it; the explicit
  rule documents the requirement.
* The first rewrite keeps a mistyped SDK URL from being answered by the SPA's `index.html`
  (which would make the viewer fail with a confusing error); create any small
  `free-pdf-sdk/404.html`, or drop that rule if you prefer.
* No COOP/COEP headers are needed.

## Other servers

nginx:

```nginx
location /free-pdf-sdk/ {
  types { application/wasm wasm; }
  add_header Cache-Control "public, max-age=31536000, immutable";
  location ~ /viewer/index\.html$ { add_header Cache-Control "no-cache"; }
}
```

Netlify / Cloudflare Pages `_headers`:

```
/free-pdf-sdk/*
  Cache-Control: public, max-age=31536000, immutable
/free-pdf-sdk/*/viewer/index.html
  Cache-Control: no-cache
```

## Using it from a bundler

The wrapper is a single ES module with no dependencies. Either import the copy in your SDK
folder at runtime:

```js
const { FreePdf } = await import(/* webpackIgnore: true */ "/free-pdf-sdk/0.1.0/free-pdf-sdk.js");
```

or install the package and import it normally (`import { FreePdf } from "free-pdf-sdk"`); the
`baseUrl` still points at the deployed folder, and the wrapper and viewer versions must match
(the wrapper logs a warning when they don't).

## Vue 3 component (sketch)

```vue
<template><div ref="host" class="pdf-host" /></template>

<script>
import { markRaw } from "vue";
import { FreePdf } from "free-pdf-sdk";

export default {
  props: { pdfUrl: { type: String, required: true }, mode: { type: String, default: "view" }, author: String, dark: Boolean },
  emits: ["loaded", "error", "annotations-changed", "dirty"],
  data: () => ({ viewer: null }),
  async mounted() {
    try {
      // markRaw: the viewer is a plain object with private state; don't make it reactive.
      this.viewer = markRaw(
        await FreePdf.init(this.$refs.host, {
          baseUrl: "/free-pdf-sdk/0.1.0/",
          mode: this.mode,
          author: this.author,
          theme: this.dark ? "dark" : "light",
          document: { url: this.pdfUrl },
          listeners: {
            documentLoaded: (info) => this.$emit("loaded", info),
            annotationsChanged: (e) => this.$emit("annotations-changed", e.count),
            dirtyChanged: (e) => this.$emit("dirty", e.dirty),
          },
        }),
      );
    } catch (e) {
      this.$emit("error", e);
    }
  },
  watch: {
    pdfUrl(url) { this.viewer?.load({ url }).catch((e) => this.$emit("error", e)); },
    dark(d) { this.viewer?.setTheme(d ? "dark" : "light"); },
    mode(m) { this.viewer?.setMode(m); },
    author(a) { if (a) this.viewer?.setAuthor(a); },
  },
  beforeUnmount() { this.viewer?.destroy(); },
};
</script>

<style scoped>.pdf-host { width: 100%; height: 100%; }</style>
```

