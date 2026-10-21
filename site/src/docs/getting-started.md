---
title: Getting started
description: Install @mecorelabs/free-pdf-sdk, serve the viewer files from your site and open your first PDF in a few lines of JavaScript.
section: Get started
order: 20
---

# Getting started

Setting up Free PDF SDK takes three steps: install the package, put the viewer files where your
site serves them, and call `FreePdf.init` with the URL of that folder.

## 1. Install the package

```bash
npm install @mecorelabs/free-pdf-sdk
npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The first command installs the JavaScript wrapper and its framework integrations. The second
copies the viewer files out of the package into a folder that your web server or dev server
serves as static files. Use the public (static) folder of your build tool: `public/` for Vite,
Next.js, Create React App and Vue CLI, `static/` for SvelteKit, `src/assets/` or `public/` for
Angular, depending on your version.

`copy-viewer` writes:

```text
public/free-pdf-sdk/
  viewer/
    index.html               the page the iframe loads
    fps-viewer-<hash>.js     JavaScript glue (content-hashed)
    fps-viewer-<hash>_bg.wasm the viewer (content-hashed)
  version.json
  NOTICE  LICENSE-MIT  LICENSE-APACHE
  third-party/               licences of the open-source code the viewer embeds
```

Run `copy-viewer` again after every install or upgrade, so the viewer always matches the
wrapper. The simplest way is a script in your `package.json`:

```json
{
  "scripts": {
    "postinstall": "free-pdf-sdk copy-viewer public/free-pdf-sdk"
  }
}
```

The command removes the previous `viewer/` folder before copying, so old hashed files don't pile
up. Many teams add the folder to `.gitignore` because it is regenerated on install.

### Without npm: the release zip

Each release also ships as a zip. Unpack it and copy its folder (`free-pdf-sdk-<version>/`) into
your site, for example as `public/free-pdf-sdk/`. Besides `viewer/` and the licences it holds the
wrapper as plain ES modules (`free-pdf-sdk.js`, `element.js`, ...) with TypeScript declarations,
so you can import it straight from that folder without a bundler.

## 2. Serve the folder from your own origin

The folder's URL is your `baseUrl`. With the command above and a dev server that serves
`public/` at the root, that is `/free-pdf-sdk/`.

Two rules matter:

- **Same origin.** The folder must be served from the same scheme, host and port as the page
  that shows the viewer. `FreePdf.init` rejects a `baseUrl` on another origin with
  `BAD_REQUEST`, so you can't load the viewer from a CDN or another domain.
- **Correct MIME type.** `.wasm` files must be served as `application/wasm`. Current dev servers
  and most hosts do this already.

Check it by opening `/free-pdf-sdk/version.json` in the browser: you should get a small JSON
object with the SDK version, not your app's own `index.html` or a 404.
[Hosting and security](/docs/hosting-and-security/) covers headers, caching and
Content-Security-Policy for production.

## 3. Show your first PDF

Give the viewer an element with a size. The viewer fills it.

```html
<div id="viewer" style="height: 80vh"></div>
<script type="module" src="/src/main.ts"></script>
```

```ts
// src/main.ts
import { FreePdf, FreePdfError } from "@mecorelabs/free-pdf-sdk";

async function start(): Promise<void> {
  try {
    const viewer = await FreePdf.init(document.getElementById("viewer")!, {
      baseUrl: "/free-pdf-sdk/",
      mode: "annotate",
      author: "Ada Lovelace",
      theme: "system",
      document: { url: "/files/contract.pdf" },
    });

    viewer.on("annotationsChanged", ({ count }) => {
      console.log(`The document has ${count} comments`);
    });
    viewer.on("dirtyChanged", ({ dirty }) => {
      document.title = dirty ? "* Contract" : "Contract";
    });
  } catch (e) {
    if (e instanceof FreePdfError) console.error(`Viewer failed: ${e.code}`, e.message);
    else throw e;
  }
}

void start();
```

`FreePdf.init` resolves once the viewer is ready and the document is open. If either fails it
rejects with a `FreePdfError` and removes the iframe, so you can show your own message in its
place.

The document can also be bytes you already have, such as a file the user picked:

```ts
const input = document.querySelector<HTMLInputElement>("#file")!;
input.addEventListener("change", async () => {
  const file = input.files?.[0];
  if (file) await viewer.load({ data: file }); // the name comes from the File
});
```

### Without a bundler

With the release zip copied to `/free-pdf-sdk/`, import the wrapper from the same folder:

```html
<div id="viewer" style="height: 80vh"></div>
<script type="module">
  import { FreePdf } from "/free-pdf-sdk/free-pdf-sdk.js";

  const viewer = await FreePdf.init(document.getElementById("viewer"), {
    baseUrl: "/free-pdf-sdk/",
    mode: "view",
    document: { url: "/files/contract.pdf" },
  });
  console.log("Pages:", await viewer.getPageCount());
</script>
```

## 4. Keep the user's work

Comments are easiest to store as XFDF, separately from the PDF (more in
[Annotations and XFDF](/docs/annotations-and-xfdf/)):

```ts
viewer.on("annotationsChanged", async () => {
  const xfdf = await viewer.exportXfdf();
  await fetch("/api/documents/42/comments", { method: "PUT", body: xfdf });
});

// Next time, after the document opened:
const saved = await fetch("/api/documents/42/comments").then((r) => r.text());
await viewer.importXfdf(saved, { replace: true });
```

To get the whole PDF with every change, call `viewer.save()` (it resolves to a `Uint8Array`), or
listen for `saveRequested`, which fires when the user presses Save inside the viewer:

```ts
viewer.on("saveRequested", async ({ name, data }) => {
  await fetch(`/api/files/${encodeURIComponent(name)}`, { method: "PUT", body: data });
});
```

## Use a framework

Each integration wraps the same API and turns prop changes into the matching calls.

| Framework | Import | Page |
|---|---|---|
| React (17–19, Next.js client components) | `import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/react"` | [React](/docs/react/) |
| Vue 3 (and Nuxt inside `<ClientOnly>`) | `import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/vue"` | [Vue](/docs/vue/) |
| Svelte 4 and 5, SvelteKit | `import { freePdf } from "@mecorelabs/free-pdf-sdk/svelte"` | [Svelte](/docs/svelte/) |
| Angular | `import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element"` | [Angular](/docs/angular/) |
| Lit, Solid, plain HTML, any other | `import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element"` | [Web Component](/docs/web-component/) |
| No framework | `import { FreePdf } from "@mecorelabs/free-pdf-sdk"` | [Plain JavaScript](/docs/vanilla-js/) |

For example, in React:

```tsx
import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/react";

export function Contract() {
  return (
    <FreePdfViewer
      baseUrl="/free-pdf-sdk/"
      mode="annotate"
      document={{ url: "/files/contract.pdf" }}
      onAnnotationsChanged={({ count }) => console.log(count)}
      style={{ height: "80vh" }}
    />
  );
}
```

## Checklist of common mistakes

- [ ] **The container has no height.** The viewer fills its element; a `div` with no height
      shows nothing (the framework components fall back to a 200 px minimum). Set a height such
      as `80vh`, or make the container a flex child that grows.
- [ ] **`baseUrl` is on another origin**, for example a CDN or an API domain. `init` rejects it
      with `BAD_REQUEST`. Serve the folder from your app's own origin.
- [ ] **`baseUrl` points at the wrong folder.** It must be the folder that contains `viewer/`,
      not `viewer/` itself. If your single-page app answers unknown URLs with its own
      `index.html`, a wrong `baseUrl` loads your app inside the iframe and `init` fails with
      `TIMEOUT` after 30 seconds.
- [ ] **The viewer files are out of date.** After upgrading `@mecorelabs/free-pdf-sdk`, run `copy-viewer`
      again. The console warns "the wrapper is X but the viewer is Y" when they differ.
- [ ] **The PDF URL is on another origin without CORS.** The wrapper fetches `document.url` from
      your page, so the server holding the PDF must allow your origin (or pass the bytes with
      `{ data }`). This fails with `FETCH_FAILED`.
- [ ] **The viewer is rendered on the server.** In Next.js, Nuxt, SvelteKit or Angular SSR,
      mount it in the browser only. It needs `window`, WebGL and an iframe.
- [ ] **Read-only by surprise.** The default mode is `view`. Pass `mode: "annotate"` (or
      another mode) to allow changes; `importXfdf` and `deleteAllAnnotations` reject with
      `READ_ONLY` when no annotation feature is on.
- [ ] **Not awaiting calls.** Every method is async and rejects with a `FreePdfError` whose
      `code` tells you what went wrong.

Next: choose what your users can do in [Configuration](/docs/configuration/), and see the
[API reference](/docs/api/) for every method and event.
