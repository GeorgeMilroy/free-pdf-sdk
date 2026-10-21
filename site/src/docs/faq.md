---
title: FAQ and troubleshooting
description: Answers to common problems - a blank viewer, TIMEOUT errors, large files, mobile, server-side rendering frameworks, licensing and where documents go.
section: Reference
order: 80
---

# FAQ and troubleshooting

## The viewer is blank or never appears

Work through these in order; the browser's developer console usually names the cause.

1. **The container has no height.** The viewer fills the element you mount it in. An element
   with no height shows nothing, or only 200 px with the framework components. Give it an
   explicit height (`height: 80vh`, `100dvh`, or a flex child with `flex: 1; min-height: 0`).
2. **`baseUrl` is on another origin.** `init` rejects with `BAD_REQUEST` and a message such as
   "the SDK must be served from this page's origin". Serve the viewer folder from your app's
   own origin; it can't be loaded from a CDN.
3. **`baseUrl` points at the wrong folder.** It must be the folder that contains `viewer/` and
   `version.json`. Open `<baseUrl>/version.json` in the browser: you should see the SDK version,
   not your app's HTML or a 404. A relative `baseUrl` is resolved against the page's URL, so
   prefer one that starts with `/`.
4. **The viewer files are missing or stale.** Run `npx @mecorelabs/free-pdf-sdk copy-viewer <folder>` again
   after every install or upgrade. A warning "the wrapper is X but the viewer is Y" in the
   console means the two don't match.
5. **WebGL is unavailable.** The viewer draws with WebGL. Open
   `chrome://gpu` in Chrome or Edge, or try another browser; some virtual machines, remote desktops and
   locked-down machines disable it. Without WebGL the viewer can't start, and `init` fails with
   `INTERNAL`.
6. **A Content-Security-Policy blocks it.** The console reports a CSP violation. The viewer
   needs `'wasm-unsafe-eval'` and its inline script and style allowed;
   [Hosting and security](/docs/hosting-and-security/) shows a policy that works.
7. **The `.wasm` is served with the wrong type or not at all.** Look in the Network tab for the
   `fps-viewer-<hash>_bg.wasm` request: it should be a 200 with `application/wasm`.

## The right-hand rail looks crowded

The rail stacks a button per panel at the top and the zoom and page controls at the bottom. Below
roughly 700 px of height the two groups overlap. Give the viewer more height, or trim the rail:

```ts
ui: {
  zoomControls: false,          // or pageNavigation: false
  panels: { layers: false, attachments: false, tags: false },
}
```

## `init` fails with `TIMEOUT`

`init` waits 30 seconds for the viewer to start. When it doesn't, `init` rejects with
`TIMEOUT` and the message "the viewer did not start (is baseUrl right, and WebGL available?)".
The usual causes:

- The iframe loaded something other than the viewer: a wrong `baseUrl` that your single-page
  app answered with its own `index.html`. Make missing SDK files return 404 (see
  [Hosting and security](/docs/hosting-and-security/)).
- A CSP blocked the viewer's script or WebAssembly. (Missing WebGL usually gives `INTERNAL`
  instead: the viewer loads but can't start.)
- A very slow connection on the first visit, while the browser downloads the `.wasm` (about
  10 MB compressed). Serve it compressed and cached.

## A method call fails with `TIMEOUT`

Each request waits `timeoutMs` (default 60 000 ms) for a reply; `load` and `save` get twice
that. A request that timed out is dropped by the viewer, so it never takes effect later.

- **Background tabs.** Browsers slow down hidden tabs, and the viewer runs requests between
  frames. Retry when the tab is visible again (`document.visibilityState === "visible"`).
- **Large documents.** Raise `timeoutMs` so loads and saves have time:

  ```ts
  await FreePdf.init(el, { baseUrl: "/free-pdf-sdk/", timeoutMs: 180_000, document: { url: "/files/drawings.pdf" } });
  ```

## Large files

- A PDF can be up to 512 MiB; larger files reject with `TOO_LARGE`. In practice the browser's
  memory is the limit long before that, especially on phones.
- The whole document is held in memory: the wrapper reads it completely before handing it to
  the viewer.
  For big downloads, fetch it yourself to show progress, then pass the bytes:

  ```ts
  const response = await fetch(url);
  const data = await response.arrayBuffer(); // or read response.body to report progress
  await viewer.load({ data, name: "drawings.pdf" });
  ```

- Only one load runs at a time; a second `load` while one is running rejects with `BUSY`.
- An XFDF passed to `importXfdf` can be up to 32 MiB.

## Does it work on phones and tablets?

The viewer runs in mobile browsers that support WebAssembly and WebGL, in the same iframe as on
the desktop. Keep in mind:

- The first visit downloads about 10 MB (compressed); cache headers make later visits load from
  the browser cache.
- Memory is tighter than on desktops, so very large PDFs may fail to open.
- Give the container a height that fits the screen, such as `height: 100dvh`, and consider
  `ui` options (for example `quickToolbar: false` or `rightRail: false`) or a simpler mode to
  leave more room for the page. See [Configuration](/docs/configuration/).

Test on the devices your users have.

## Next.js, Nuxt, SvelteKit, Angular SSR

The viewer needs `window`, an iframe and WebGL, so it can only be mounted in the browser.

- **Next.js**: use the React component in a client component (`"use client"` at the top of the
  file). The component mounts the viewer in an effect, so it never runs during server
  rendering.

  ```tsx
  "use client";
  import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/react";

  export default function Viewer({ url }: { url: string }) {
    return <FreePdfViewer baseUrl="/free-pdf-sdk/" document={{ url }} style={{ height: "80vh" }} />;
  }
  ```

  Put the viewer folder in `public/free-pdf-sdk`.
- **Nuxt**: wrap the Vue component in `<ClientOnly>`, and put the folder in `public/`.
- **SvelteKit**: works with server rendering on; actions only run in the browser. See
  [Svelte](/docs/svelte/). Put the folder in `static/`.
- **Angular with SSR, or any server-rendered page using the Web Component**: import
  `@mecorelabs/free-pdf-sdk/element` only in the browser, for example with a dynamic `import()` after
  checking `isPlatformBrowser`, because the element class needs browser globals when the module
  loads.

## Can I show several viewers on one page?

Yes. Each `FreePdf.init` creates its own iframe with its own private message channel, so viewers
never interfere with each other. Each one loads its own copy of the viewer in memory, so keep the
number reasonable on phones, and call `destroy()` (or unmount the component) when you no longer
need one.

## Can I load the viewer from a CDN?

No. The viewer folder must be served from the same origin as your page. The SDK has no CDN build
and no server component: you serve the static files yourself. The reasons are explained in
[Hosting and security](/docs/hosting-and-security/).

## Where do my documents go?

Nowhere. Your page fetches the PDF (or passes its bytes), the viewer opens it with WebAssembly
in the browser, and the result of `save()`, `exportXfdf()` or the `saveRequested` event goes to
your code. The SDK sends no documents, comments, passwords or telemetry to any server. The viewer
keeps its own preferences in the browser's storage for your origin.

## The viewer is read-only, but I passed annotations

The default mode is `view`. Choose a mode that edits, such as `annotate`. Any configuration with
no editing feature is read-only, and in that state `importXfdf` and `deleteAllAnnotations`
reject with `READ_ONLY`. Check what the viewer resolved:

```ts
const config = await viewer.getConfig();
console.log(config.mode, config.readOnly, config.features);
```

If the configuration allows comments but you get `NOT_PERMITTED`, the document's own security
forbids them; see [Encrypted PDFs](/docs/encrypted-pdfs/).

## My PDF URL fails with `FETCH_FAILED`

The wrapper fetches `document.url` from your page. A PDF on another origin needs CORS headers
that allow your origin; private files may need `fetchInit` with credentials or headers:

```ts
await viewer.load({
  url: "https://files.example.com/contract.pdf",
  fetchInit: { credentials: "include" },
});
```

An HTTP error status also gives `FETCH_FAILED`, with the status in the message.

## Can I open a password-protected PDF?

Yes. Pass `password` to `init` or `load`, and lift owner restrictions with `unlock`. See
[Encrypted PDFs](/docs/encrypted-pdfs/).

## What does it not do?

Free PDF SDK has no server component, no hosted or CDN version, no document comparison and no
colour separation preview. The full list of what it offers is in
[Configuration](/docs/configuration/).

## What is the licence?

Free PDF SDK is licensed **MIT OR Apache-2.0**, at your option, including commercial use and
white-labelling. The viewer embeds open-source components (it is built on the pdfcraft engine);
their licences and notices are in `NOTICE` and `third-party/`, which `copy-viewer` copies next to
the viewer. Keep those files when you deploy or redistribute it.

## Does the viewer support PDF layers?

Yes. Documents with optional content (layers such as structure, furniture, electrical, dimensions or revisions)
get a Layers panel on the right-hand rail. Each layer has a show/hide toggle, the page redraws
immediately, and printing follows what is shown. Toggling a layer does not change the file. Hide
the panel with `ui: { panels: { layers: false } }` if you don't want it; the sample document in
the [live demo](/demo/) has four layers to try.
