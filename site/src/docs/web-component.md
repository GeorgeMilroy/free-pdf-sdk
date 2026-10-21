---
title: Web Component
description: Use the <free-pdf-viewer> custom element in any page or framework, with its attributes, properties, fps-* events and API access.
section: Frameworks
order: 40
---

# Web Component

`<free-pdf-viewer>` is Free PDF SDK as a standard custom element. It works anywhere HTML does:
plain pages, Angular, Lit, Solid, Svelte, server-rendered templates. Attributes and properties
configure the viewer, DOM events report what happens, and `whenReady()` hands you the full
JavaScript API.

```html
<free-pdf-viewer
  base-url="/free-pdf-sdk/"
  src="/files/contract.pdf"
  mode="annotate"
  features='{"stamps": false}'
  style="height: 80vh"
></free-pdf-viewer>
```

## Install and register

Install the package and copy the viewer files into a folder your site serves:

```bash
npm install @mecorelabs/free-pdf-sdk
npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies `viewer/`, `version.json` and the licences into
`public/free-pdf-sdk`. Run it again after every upgrade (a `postinstall` or `prebuild` script
works well) so the viewer always matches the wrapper. The folder's URL is your `base-url`, and it
must be on the same origin as the page.

Register the element once, before it is used:

```ts
import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element";

defineFreePdfViewer(); // registers <free-pdf-viewer>
```

`defineFreePdfViewer()` is safe to call more than once. Pass a name to register the element under
another tag, for example `defineFreePdfViewer("pdf-viewer")`.

### Without a bundler

The release zip contains the same modules next to the `viewer/` folder. Serve the unzipped
folder (for example at `/free-pdf-sdk/`) and import the element from it:

```html
<script type="module">
  import { defineFreePdfViewer } from "/free-pdf-sdk/element.js";
  defineFreePdfViewer();
</script>
```

## Give it a height

The element is `display: block` with `position: relative` and a `min-height` of 200px; the
viewer fills it. Set a height, or the viewer is only 200px tall:

```html
<style>
  free-pdf-viewer { height: calc(100vh - 64px); }
</style>
```

Any CSS that sizes a block element works: `height`, `flex: 1` inside a flex column, a grid area.

## Attributes

| Attribute | Value | Becomes |
|---|---|---|
| `base-url` | URL of the viewer folder, on the page's origin. Required: nothing mounts without it. | `baseUrl` |
| `src` | URL of a PDF to open (same-origin, or a CORS-enabled URL). | `document: { url }` |
| `mode` | `view`, `review`, `annotate`, `fill`, `sign`, `organize`, `edit` or `full`. Default `view` (read-only). | `mode` |
| `features` | JSON object of feature switches on top of the mode, such as `{"stamps": false}`. | `features` |
| `ui` | JSON object of interface options, such as `{"toolsPanel": false}`. | `ui` |
| `disabled-elements` | Element ids separated by commas or spaces, such as `download print tool:stamp`. | `disabledElements` |
| `theme` | `light`, `dark` or `system` (default). | `theme` |
| `author` | Author name of new comments. | `author` |
| `locale` | Interface language, such as `en`, `de` or `zh-Hans`. | `locale` |
| `password` | Password used when `src` is opened (opens a protected file and lifts owner restrictions). | `password` |
| `loader` | URL of a loading image (SVG, GIF, animated WebP or PNG), or `none`. Read when the viewer mounts. | `loader` |

If `features` or `ui` is not valid JSON, the element dispatches an `fps-error` event with code
`BAD_REQUEST` and ignores the attribute.

Changing an attribute later calls the matching API method on the running viewer (`mode` calls
`setMode`, `features` calls `setFeatures`, `src` opens the new document, and so on). Changes made
in the same task are batched. Changing `base-url` destroys the viewer and mounts a new one.

```js
const el = document.querySelector("free-pdf-viewer");
el.setAttribute("mode", "review");
el.setAttribute("disabled-elements", "download, print");
```

Two details follow from how changes map to calls:

- `features` replaces the overrides: removing the attribute calls `setFeatures({})`, which goes
  back to the mode's own features. `ui` options you leave out keep their current value.
- `password` is only read when a document is opened. Changing it alone does nothing; it is used
  the next time `src` changes.

## Properties

The element has properties for the options you are most likely to set from code:

| Property | Type | Notes |
|---|---|---|
| `baseUrl` | `string` | Same as `base-url`. |
| `mode` | `Mode` | Same as `mode`. |
| `features` | `FeatureFlags` | An object; no JSON needed. |
| `ui` | `UiOptions` | An object; no JSON needed. |
| `theme` | `Theme` | Same as `theme`. |
| `options` | `Partial<ViewerOptions>` | Sets several options at once (see below). |
| `viewer` | `FreePdfViewer \| null` | Read-only. The viewer's API once it is ready, else `null`. |

```js
const el = document.querySelector("free-pdf-viewer");
el.features = { stamps: false, measurement: false };
el.ui = { quickToolbar: false, panels: { bookmarks: false } };
```

A value set through a property wins over the attribute of the same name, and stays in force:
once you set `el.mode`, later changes to the `mode` attribute are ignored.

### The `options` setter

`options` takes any of the `FreePdf.init` options except `listeners`: `baseUrl`, `document`,
`mode`, `features`, `ui`, `theme`, `author`, `locale`, `password`, `disabledElements`,
`loader` and `timeoutMs`. What you pass is merged into what was set before, and it wins over attributes.

Use it for everything that has no property of its own, most usefully a document given as bytes:

```js
const input = document.querySelector("input[type=file]");
input.addEventListener("change", () => {
  const file = input.files[0];
  if (!file) return;
  el.options = {
    document: { data: file, name: file.name },
    author: "Priya Raman",
    disabledElements: ["download", "print"],
  };
});
```

Reading `el.options` returns a copy of the options set through properties (not the attributes).

## Events

The element dispatches a DOM `CustomEvent` for every viewer event. The event's data is in
`detail`. The events bubble and cross shadow roots (`bubbles` and `composed`).

| DOM event | `detail` | When |
|---|---|---|
| `fps-ready` | `{ sdk, pdfcraftRev }` | The viewer is configured and ready for requests. |
| `fps-document-loaded` | `{ pageCount, permissions }` | A document opened. |
| `fps-document-error` | `{ code, message }` | A document could not be opened. |
| `fps-annotations-changed` | `{ count }` | Comments were added, changed or removed, by the user or the API. |
| `fps-dirty-changed` | `{ dirty }` | The document gained or lost unsaved changes. |
| `fps-error` | `{ code, message }` | Something failed outside a request: the viewer could not start, an option change was refused, an attribute held invalid JSON. |
| `fps-save-requested` | `{ name, saveAs, data }` | The user pressed Save in the viewer. `data` is the saved PDF as a `Uint8Array`; the viewer now treats it as saved. |

`permissions` is `{ canAnnotate, canPrint, canCopy, isEncrypted, canUnlock }`.

```js
el.addEventListener("fps-document-loaded", (e) => {
  console.log(`${e.detail.pageCount} pages`, e.detail.permissions);
});

el.addEventListener("fps-annotations-changed", (e) => {
  document.querySelector("#count").textContent = String(e.detail.count);
});

el.addEventListener("fps-save-requested", async (e) => {
  const { name, data } = e.detail;
  await fetch(`/api/files/${encodeURIComponent(name)}`, { method: "PUT", body: data });
});

el.addEventListener("fps-error", (e) => {
  console.error(e.detail.code, e.detail.message);
});
```

Add listeners before the element is connected (or in the same task) to hear `fps-ready` and the
first `fps-document-loaded`.

## Call the API

For everything else (export comments, save, navigate) use the viewer's API. `whenReady()`
resolves with it once the viewer is ready and the document from `src` is open; `viewer` returns
it synchronously, or `null` before then.

```js
const el = document.querySelector("free-pdf-viewer");

document.querySelector("#export").addEventListener("click", async () => {
  const viewer = await el.whenReady();
  const xfdf = await viewer.exportXfdf();
  await fetch("/api/annotations/contract-42", {
    method: "PUT",
    headers: { "Content-Type": "application/vnd.adobe.xfdf" },
    body: xfdf,
  });
});
```

`whenReady()` can be called at any time, even before the element is connected: it resolves
once the viewer is ready and its first document has loaded (or failed to), or rejects if it can't start.

```js
const el = document.createElement("free-pdf-viewer");
el.setAttribute("base-url", "/free-pdf-sdk/");
el.setAttribute("src", "/files/contract.pdf");
el.style.height = "80vh";
document.querySelector("main").append(el);

const viewer = await el.whenReady();
console.log(await viewer.getPageCount());
```

Calls made through the API don't update the element's attributes or properties. If you call
`viewer.setMode("review")` and later change the `mode` attribute, the attribute's value is
applied then.

## A complete page

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Contract review</title>
    <style>
      body { margin: 0; display: flex; flex-direction: column; height: 100vh; font-family: system-ui, sans-serif; }
      header { display: flex; gap: 12px; align-items: center; padding: 8px 16px; }
      free-pdf-viewer { flex: 1; }
    </style>
  </head>
  <body>
    <header>
      <select id="mode">
        <option value="view">View</option>
        <option value="review">Review</option>
        <option value="annotate" selected>Annotate</option>
      </select>
      <span><span id="count">0</span> comments</span>
      <button id="export">Save comments</button>
    </header>

    <free-pdf-viewer
      base-url="/free-pdf-sdk/"
      src="/files/contract.pdf"
      mode="annotate"
      author="Priya Raman"
      disabled-elements="download"
    ></free-pdf-viewer>

    <script type="module">
      import { defineFreePdfViewer } from "/free-pdf-sdk/element.js";
      defineFreePdfViewer();

      const el = document.querySelector("free-pdf-viewer");

      el.addEventListener("fps-annotations-changed", (e) => {
        document.querySelector("#count").textContent = String(e.detail.count);
      });
      el.addEventListener("fps-error", (e) => console.error(e.detail));

      document.querySelector("#mode").addEventListener("change", (e) => {
        el.setAttribute("mode", e.target.value);
      });

      document.querySelector("#export").addEventListener("click", async () => {
        const viewer = await el.whenReady();
        const xfdf = await viewer.exportXfdf();
        await fetch("/api/annotations/contract", { method: "PUT", body: xfdf });
      });
    </script>
  </body>
</html>
```

## Remove it

Removing the element from the DOM destroys its viewer: the iframe goes away and pending calls
reject with `DESTROYED`. Inserting the element again mounts a new viewer.

## Things to know

- The viewer runs in an iframe served from `base-url`, on your page's origin, and talks to the
  page over `postMessage`. A `base-url` on another origin is refused with `BAD_REQUEST`.
- PDFs are processed in the browser with WebAssembly; the SDK does not upload them anywhere.
- The viewer needs WebGL. If it can't start, `fps-error` reports `TIMEOUT` (usually a wrong
  `base-url`) or `INTERNAL` (the viewer loaded but could not start, for example without WebGL).
- If a document can't be opened, `fps-document-error` reports why and the viewer keeps running:
  set another `src` (or `password`) to try again.
- TypeScript: the element class is exported as `FreePdfViewerElement` from
  `@mecorelabs/free-pdf-sdk/element`, and the DOM event names as `ELEMENT_EVENTS`.

[Configuration](/docs/configuration/) lists every mode, feature, `ui` option and
`disabled-elements` id; the [API reference](/docs/api/) lists every method of the viewer.

Using a framework? See [Angular](/docs/angular/) and [Svelte](/docs/svelte/), or the dedicated
[React](/docs/react/) and [Vue](/docs/vue/) components.
