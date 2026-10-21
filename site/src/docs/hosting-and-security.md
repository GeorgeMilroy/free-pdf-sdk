---
title: Hosting and security
description: Serve the viewer files correctly in production - same origin, MIME types, caching, Content-Security-Policy and examples for Firebase Hosting, Netlify, Nginx and Express.
section: Guides
order: 53
---

# Hosting and security

The viewer is a folder of static files. Any host that serves static files can serve it, as long
as it follows four rules: same origin, the right MIME type for `.wasm`, sensible caching, and a
Content-Security-Policy (if you use one) that allows WebAssembly.

## What you deploy

After `npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk` (or unpacking the release zip), the
folder looks like this:

```text
free-pdf-sdk/
  viewer/
    index.html                 not hashed: names the two files below
    fps-viewer-<hash>.js       JavaScript glue, content-hashed
    fps-viewer-<hash>_bg.wasm  the viewer, content-hashed
  version.json                 { "sdk": "...", "pdfcraftRev": "..." }
  NOTICE  LICENSE-MIT  LICENSE-APACHE
  third-party/
```

The `.wasm` file is the large one: about 10 MB compressed. Serve it
compressed (brotli or gzip) and cache it; see below.

## Serve the folder from the page's origin

`baseUrl` must have the same scheme, host and port as the page that calls `FreePdf.init`:

```ts
// Page: https://app.example.com/projects/42
FreePdf.init(el, { baseUrl: "/free-pdf-sdk/" });                         // works
FreePdf.init(el, { baseUrl: "https://app.example.com/free-pdf-sdk/" });  // works
FreePdf.init(el, { baseUrl: "https://cdn.example.com/free-pdf-sdk/" });  // BAD_REQUEST
```

Why this is required:

- **Strict message checks.** The page and the viewer exchange requests and replies with
  `postMessage`. The wrapper accepts a message only from the viewer's own iframe on the SDK's
  origin, and the viewer accepts requests only from a parent on its own origin. Keeping both on
  one origin lets each side reject everything else.
- **Isolation without cross-origin plumbing.** Each viewer is its own iframe, so its interface
  can't be affected by your page's CSS, and several viewers on a page never share state. A
  same-origin iframe needs no CORS, COOP or COEP configuration.
- **Your network rules apply.** The wrapper fetches `document.url` in your page, with your
  cookies, headers and CORS rules, then hands the bytes to the viewer.

The PDF itself can come from any origin your page may `fetch` from (for example a signed storage
URL with CORS configured), or you can pass bytes with `{ data }`.

## Required headers

| What | Requirement |
|---|---|
| `.wasm` files | `Content-Type: application/wasm`. With another type the browser can't stream-compile the module and falls back to a slower load. |
| `viewer/index.html` | Must be served as HTML and be frameable by your own pages: no `X-Frame-Options: DENY` (`SAMEORIGIN` is fine); with CSP `frame-ancestors`, allow `'self'`. |
| Missing files | Must return 404, not your single-page app's `index.html`. Otherwise a wrong `baseUrl` or a stale file name loads your app inside the iframe and `init` fails with `TIMEOUT`. |
| COOP / COEP | Not needed. The viewer uses no `SharedArrayBuffer` or threads. Don't add `Cross-Origin-Embedder-Policy` for the SDK: it would also block cross-origin PDFs your page fetches. |

## Cache hashed files

The JavaScript glue and the `.wasm` have a content hash in their names, so a new version always
has new names. Cache them forever, and keep the two unhashed files fresh:

| Files | `Cache-Control` |
|---|---|
| `viewer/fps-viewer-<hash>.js`, `viewer/fps-viewer-<hash>_bg.wasm` | `public, max-age=31536000, immutable` |
| `viewer/index.html`, `version.json` | `no-cache` (revalidated on each load; it names the current hashed files) |
| Licence files | anything; they are not loaded by the viewer |

After the first visit the browser loads the viewer from its cache, so later visits don't download
the `.wasm` again.

If you use the release zip and import `free-pdf-sdk.js` from the folder, that file isn't hashed:
either serve it with `no-cache`, or put the version in the folder name (`/free-pdf-sdk/0.2.0/`)
and cache the whole folder as immutable.

## Content-Security-Policy

The viewer's `index.html` contains one small inline `<script type="module">` that loads the
hashed files, and one inline `<style>`. Both change with every release, so a CSP for the viewer
folder allows them by hash. WebAssembly needs `'wasm-unsafe-eval'`.

This script prints a policy for the copy you serve. Run it after each `copy-viewer`, for example
in the same `postinstall` script, and put the output in your host's header configuration:

```js
// scripts/viewer-csp.mjs
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const file = process.argv[2] ?? "public/free-pdf-sdk/viewer/index.html";
const html = readFileSync(file, "utf8");
const sha = (text) => `'sha256-${createHash("sha256").update(text, "utf8").digest("base64")}'`;
const script = html.match(/<script type="module">([\s\S]*?)<\/script>/)[1];
const style = html.match(/<style>([\s\S]*?)<\/style>/)[1];

console.log(
  [
    "default-src 'self'",
    `script-src 'self' 'wasm-unsafe-eval' ${sha(script)}`,
    `style-src 'self' ${sha(style)}`,
    "img-src 'self' data: blob:",
    "object-src 'none'",
    "base-uri 'none'",
    "frame-ancestors 'self'",
  ].join("; "),
);
```

```bash
node scripts/viewer-csp.mjs public/free-pdf-sdk/viewer/index.html
```

Apply that policy to `/free-pdf-sdk/viewer/*` only. If your browser console reports a blocked
resource while you print or save from the viewer, add the source it names (such as `blob:`) to the
directive it names.

Your **host page's** CSP needs:

| Directive | Why |
|---|---|
| `frame-src 'self'` (or `child-src 'self'`) | The viewer is a same-origin iframe. |
| `connect-src 'self' <PDF origins>` | The wrapper fetches `document.url` from your page. List every origin your PDFs come from. |
| `script-src` | Whatever already allows your own bundle; the wrapper is ordinary JavaScript with no `eval`. |

## Hosting examples

All examples assume the folder is published at `/free-pdf-sdk/`.

### Firebase Hosting

Firebase Hosting serves `.wasm` as `application/wasm` and compresses responses. Add cache rules,
and keep the single-page-app rewrite away from the SDK folder:

```json
{
  "hosting": {
    "public": "dist",
    "headers": [
      {
        "source": "/free-pdf-sdk/**",
        "headers": [{ "key": "Cache-Control", "value": "no-cache" }]
      },
      {
        "source": "/free-pdf-sdk/viewer/fps-viewer-*",
        "headers": [{ "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }]
      },
      {
        "source": "**/*.wasm",
        "headers": [{ "key": "Content-Type", "value": "application/wasm" }]
      }
    ],
    "rewrites": [
      { "source": "!/free-pdf-sdk/**", "destination": "/index.html" }
    ]
  }
}
```

- When several rules set the same header for a file, Firebase applies the last one, so the
  `immutable` rule for hashed files comes after the folder rule.
- Rewrites only apply to files that don't exist. The `!` glob keeps the single-page-app rewrite
  away from the SDK folder, so a mistyped SDK URL gets a real 404 instead of your app's
  `index.html`.
- Leave `cleanUrls` off (the default). With it on, Firebase redirects `viewer/index.html` to
  `viewer`, and the viewer's relative script paths stop resolving.
- With Vue CLI or Vite, `public/` is copied into `dist/`, so `copy-viewer public/free-pdf-sdk`
  ends up at `/free-pdf-sdk/`.

### Netlify

Add a `_headers` file to your publish folder (for Vite, `public/_headers`). The rules don't
overlap, so each file gets exactly one `Cache-Control`:

```text
/free-pdf-sdk/viewer/index.html
  Cache-Control: no-cache

/free-pdf-sdk/version.json
  Cache-Control: no-cache

/free-pdf-sdk/viewer/fps-viewer-*
  Cache-Control: public, max-age=31536000, immutable
```

Netlify serves static files before applying a single-page-app `/* /index.html 200` redirect, so
existing viewer files are not affected by it. Check the `.wasm` content type with the `curl`
command at the end of this page.

### Nginx

```nginx
http {
  include       mime.types;
  # Recent mime.types files already map .wasm; this line covers older ones.
  types { application/wasm wasm; }

  gzip on;
  gzip_types application/wasm application/javascript text/css application/json;
  # With the brotli module installed: brotli on; brotli_types application/wasm application/javascript;

  server {
    root /var/www/app;

    location /free-pdf-sdk/ {
      add_header Cache-Control "no-cache";
      try_files $uri =404;

      location ~ ^/free-pdf-sdk/viewer/fps-viewer-[0-9a-f]+(\.js|_bg\.wasm)$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
        try_files $uri =404;
      }
    }

    location / {
      try_files $uri /index.html;
    }
  }
}
```

`try_files $uri =404` returns a real 404 for a missing SDK file instead of falling through to
the app's `index.html`.

### Express

```js
import express from "express";
import compression from "compression";

const app = express();

// Compress the .wasm too, whatever compression's default type list says.
app.use(compression({ filter: (req, res) => req.path.endsWith(".wasm") || compression.filter(req, res) }));

app.use(
  "/free-pdf-sdk",
  express.static("public/free-pdf-sdk", {
    fallthrough: false, // a missing SDK file is a 404, never the app's index.html
    setHeaders(res, path) {
      if (path.endsWith(".wasm")) res.setHeader("Content-Type", "application/wasm");
      const hashed = /[\\/]viewer[\\/]fps-viewer-[0-9a-f]+(\.js|_bg\.wasm)$/.test(path);
      res.setHeader("Cache-Control", hashed ? "public, max-age=31536000, immutable" : "no-cache");
    },
  }),
);

app.use(express.static("dist"));
app.use((req, res) => res.sendFile("index.html", { root: "dist" })); // single-page app fallback

app.listen(3000);
```

### Check your deployment

```bash
curl -sI https://app.example.com/free-pdf-sdk/viewer/index.html
curl -sI -H "Accept-Encoding: br, gzip" \
  https://app.example.com/free-pdf-sdk/viewer/fps-viewer-<hash>_bg.wasm
```

The `.wasm` response should show `content-type: application/wasm`, a `content-encoding` of `br`
or `gzip`, and the long-lived `cache-control`.

## Privacy: documents stay in the browser

- **No uploads.** The SDK has no server component. The PDF is fetched by your page (or passed as
  bytes), handed to the iframe, and opened by WebAssembly in the user's browser. Neither the
  wrapper nor the viewer sends the document, its comments or its passwords anywhere.
- **No remote code.** The iframe loads only the files in your SDK folder, from your origin. No
  CDN, analytics or telemetry is involved.
- **You decide where output goes.** `save()`, `exportXfdf()` and the `saveRequested` event hand
  data to your code; when the `save` feature is on, the viewer's Save button never triggers a
  download by itself.
- **Passwords stay on the channel.** A password you pass to `init`, `load` or `unlock` travels
  only over the same-origin message channel to the viewer.
- **Links in documents.** By default a link to a web page in a PDF may open after the user
  confirms a prompt. Set `ui: { externalLinks: false }` to turn that off.
- **Local settings.** The viewer keeps its own preferences in the browser's storage for your
  origin.

To keep users from taking a copy, turn off the `download`, `export` and `print` features (see
[Configuration](/docs/configuration/)); remember that anything shown in a browser can still be
captured by a determined user.
