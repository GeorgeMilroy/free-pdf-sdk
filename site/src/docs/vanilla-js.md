---
title: Vanilla JavaScript
description: Mount the viewer with FreePdf.init from plain ES modules or a bundler, listen to events, destroy it, and run several viewers on one page.
section: Frameworks
order: 45
---

# Vanilla JavaScript

Without a framework, call `FreePdf.init(element, options)`. It mounts the viewer in `element`,
waits until it is ready (and until `options.document` is open), and resolves with the viewer's
API.

```ts
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  document: { url: "/files/contract.pdf" },
});

viewer.on("annotationsChanged", ({ count }) => console.log(`${count} comments`));
```

## Get the files

There are two ways to get the SDK, depending on whether you use a bundler.

### With a bundler (npm)

```bash
npm install free-pdf-sdk
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

Import `FreePdf` from `free-pdf-sdk` in your code. The second command copies the viewer files
(`viewer/`, `version.json`, the licences) into `public/free-pdf-sdk`; point `baseUrl` at the URL
that folder is served from. Run it again after every upgrade so the viewer matches the wrapper
(the wrapper logs a warning when the versions differ).

### Without a bundler (release zip)

Download the release zip, unzip it, and serve the folder from your site, for example at
`/free-pdf-sdk/`. Besides `viewer/` and `version.json`, it contains the wrapper as plain ES
modules, so a page can import it directly:

```html
<script type="module">
  import { FreePdf } from "/free-pdf-sdk/free-pdf-sdk.js";
  // ...
</script>
```

`free-pdf-sdk.js` (the core API) and `element.js` (the [Web Component](/docs/web-component/))
work without a build step. The React and Vue modules in the folder import `react` and `vue`, so
they need a bundler.

### Same origin

Whichever way you choose, `baseUrl` must be on the same origin (scheme, host and port) as the
page. The viewer is an iframe loaded from that folder, and it talks to your page over
`postMessage`; `init` rejects with `BAD_REQUEST` if `baseUrl` is on another origin. The PDF itself
can come from anywhere your page can `fetch`.

The folder includes a large WebAssembly file (about 10 MB compressed). Serve `.wasm` files as
`application/wasm`, compressed if your server can; [Hosting and security](/docs/hosting-and-security/)
covers headers, caching and Content-Security-Policy.

## A complete page

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Contract</title>
    <style>
      body { margin: 0; display: flex; flex-direction: column; height: 100vh; font-family: system-ui, sans-serif; }
      header { display: flex; gap: 12px; align-items: center; padding: 8px 16px; }
      #viewer { flex: 1; min-height: 0; }
    </style>
  </head>
  <body>
    <header>
      <span id="count">0 comments</span>
      <button id="save" disabled>Save comments</button>
      <span id="status"></span>
    </header>
    <div id="viewer"></div>

    <script type="module">
      import { FreePdf, FreePdfError } from "/free-pdf-sdk/free-pdf-sdk.js";

      const status = document.getElementById("status");

      try {
        const viewer = await FreePdf.init(document.getElementById("viewer"), {
          baseUrl: "/free-pdf-sdk/",
          mode: "annotate",
          author: "Priya Raman",
          theme: "system",
          disabledElements: ["download"],
          document: { url: "/files/contract.pdf" },
          listeners: {
            annotationsChanged: ({ count }) => {
              document.getElementById("count").textContent = `${count} comments`;
            },
            error: ({ code, message }) => console.error(code, message),
          },
        });

        // Restore comments saved earlier.
        const saved = await fetch("/api/contracts/42/annotations");
        if (saved.ok) await viewer.importXfdf(await saved.text(), { replace: true });

        const button = document.getElementById("save");
        button.disabled = false;
        button.addEventListener("click", async () => {
          const xfdf = await viewer.exportXfdf();
          await fetch("/api/contracts/42/annotations", {
            method: "PUT",
            headers: { "Content-Type": "application/vnd.adobe.xfdf" },
            body: xfdf,
          });
          status.textContent = "Saved";
        });
      } catch (e) {
        status.textContent = e instanceof FreePdfError ? `Could not open the viewer (${e.code})` : String(e);
      }
    </script>
  </body>
</html>
```

## Size the container

The viewer is an iframe that fills `element` (`width: 100%; height: 100%`). Give the element a
size: a fixed height, `flex: 1` in a flex column (as above), or a grid area.

## Options

| Option | Type | Notes |
|---|---|---|
| `baseUrl` | `string` | Required. URL of the viewer folder, on the page's origin. |
| `document` | `{ url, name?, fetchInit? }` or `{ data, name? }` | Opened once the viewer is ready. `fetchInit` is passed to `fetch` (headers, credentials). `data` is an `ArrayBuffer`, `Uint8Array` or `Blob`, and is copied. |
| `mode` | `Mode` | `view` (default, read-only), `review`, `annotate`, `fill`, `sign`, `organize`, `edit` or `full`. |
| `features` | `FeatureFlags` | Feature switches on top of the mode, such as `{ stamps: false }`. |
| `ui` | `UiOptions` | Interface options, such as `{ toolsPanel: false }`. |
| `disabledElements` | `ElementId[]` | Parts of the interface to hide, such as `["download", "print"]`. |
| `author` | `string` | Author of new comments; wins over a name saved in the viewer's settings. |
| `theme` | `"light" \| "dark" \| "system"` | Default `system`. |
| `password` | `string` | Opens a document that needs one, and lifts owner restrictions. |
| `locale` | `string` | Interface language, such as `en`, `de` or `zh-Hans`; the closest available, else English. |
| `timeoutMs` | `number` | How long a request waits for a reply before rejecting with `TIMEOUT` (default 60000; loads and saves get twice that). |
| `listeners` | `{ [event]: listener }` | Attached before the viewer starts, so they hear `ready` and the first `documentLoaded`. |

After `init`, change the configuration with `setMode`, `setFeatures`, `setUi`,
`setDisabledElements`, `setTheme`, `setAuthor` and `setLocale`, and read the resolved
configuration with `getConfig()`:

```ts
await viewer.setMode("review");
await viewer.setFeatures({ markup: false }); // replaces earlier overrides
await viewer.setUi({ quickToolbar: false }); // other ui options keep their value

const config = await viewer.getConfig();
console.log(config.mode, config.features.comments, config.readOnly);
```

## Listen to events

Pass `listeners` to `init` for events that may fire while `init` is still running, and use
`on`/`off` afterwards:

```ts
const onDirty = ({ dirty }: { dirty: boolean }) => {
  document.title = dirty ? "• Contract" : "Contract";
};
viewer.on("dirtyChanged", onDirty);
// later
viewer.off("dirtyChanged", onDirty);
```

| Event | Data | When |
|---|---|---|
| `ready` | `{ sdk, pdfcraftRev }` | The viewer is configured. Listeners added later with `on` still receive it. |
| `documentLoaded` | `{ pageCount, permissions }` | A document opened. |
| `documentError` | `{ code, message }` | A document could not be opened. |
| `annotationsChanged` | `{ count }` | Comments were added, changed or removed, by the user or the API. |
| `dirtyChanged` | `{ dirty }` | The document gained or lost unsaved changes. |
| `error` | `{ code, message }` | Something failed outside a request, for example a wrong `password` at load. |
| `saveRequested` | `{ name, saveAs, data }` | The user pressed Save in the viewer. `data` is the PDF as a `Uint8Array`; the viewer now treats it as saved. |

Handle the viewer's Save button by uploading the PDF it hands you:

```ts
viewer.on("saveRequested", async ({ name, data }) => {
  await fetch(`/api/files/${encodeURIComponent(name)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/pdf" },
    body: data,
  });
});
```

To remove the button instead, turn the `save` feature off: `features: { save: false }`.

## Handle errors

Every method is async and rejects with a `FreePdfError` that has a `code` and a `message`
(English, meant for logs). `init` rejects (and removes the iframe) when the viewer can't start or
the first document can't be opened:

```ts
import { FreePdf, FreePdfError } from "free-pdf-sdk";

try {
  const viewer = await FreePdf.init(element, { baseUrl: "/free-pdf-sdk/", document: { url } });
} catch (e) {
  if (!(e instanceof FreePdfError)) throw e;
  switch (e.code) {
    case "TIMEOUT":
      // The viewer did not start: is baseUrl right?
      break;
    case "INTERNAL":
      // The viewer could not start, for example without WebGL.
      break;
    case "FETCH_FAILED":
      // The PDF could not be downloaded (HTTP error, CORS, network).
      break;
    case "NEEDS_PASSWORD":
    case "WRONG_PASSWORD":
      // Ask for a password and init again with `password`.
      break;
    case "LOAD_FAILED":
      // Not a PDF the viewer can open.
      break;
    default:
      console.error(e.code, e.message);
  }
}
```

When documents may fail to open (user uploads, for example), `init` without a `document` and
call `load` afterwards. A failed `load` rejects, and the viewer stays (if the viewer itself
couldn't open the file, the document shown before is closed):

```ts
const viewer = await FreePdf.init(element, { baseUrl: "/free-pdf-sdk/", mode: "review" });

input.addEventListener("change", async () => {
  const file = input.files?.[0];
  if (!file) return;
  try {
    const { pageCount } = await viewer.load({ data: file });
    console.log(`${file.name}: ${pageCount} pages`);
  } catch (e) {
    console.error(e);
  }
});
```

## Destroy the viewer

`destroy()` removes the iframe and its listeners. Requests still waiting reject with
`DESTROYED`. It is safe to call twice.

```ts
await viewer.destroy();
```

Call it when your page removes the container (closing a dialog, leaving a route in a single-page
app). Removing the container from the DOM without `destroy()` leaves the wrapper's `message`
listener behind.

## Several viewers on one page

Call `init` once per container. Each viewer is its own iframe with its own channel, so events
and requests never cross between them:

```html
<div class="pair">
  <div id="original"></div>
  <div id="revised"></div>
</div>

<style>
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; height: 90vh; }
</style>

<script type="module">
  import { FreePdf } from "/free-pdf-sdk/free-pdf-sdk.js";

  const [original, revised] = await Promise.all([
    FreePdf.init(document.getElementById("original"), {
      baseUrl: "/free-pdf-sdk/",
      document: { url: "/files/contract-v1.pdf" },
    }),
    FreePdf.init(document.getElementById("revised"), {
      baseUrl: "/free-pdf-sdk/",
      mode: "review",
      document: { url: "/files/contract-v2.pdf" },
    }),
  ]);

  revised.on("annotationsChanged", ({ count }) => console.log(`revised: ${count} comments`));
</script>
```

Each viewer runs its own copy of the engine, so each one costs memory. The browser downloads the
viewer files once and reuses them from its cache.

## Things to know

- PDFs are processed in the browser with WebAssembly; the SDK does not upload them anywhere.
  Whatever you send to your server (XFDF, the saved PDF) is your code's choice.
- The viewer needs WebGL.
- Any configuration without an editing feature is read-only (`view` mode, for example).
  `importXfdf` and `deleteAllAnnotations` need an annotation feature; without one they reject
  with `READ_ONLY`.
- [Configuration](/docs/configuration/) lists every mode, feature and `ui` option, and the
  [API reference](/docs/api/) every method.
- Prefer markup to code? The [Web Component](/docs/web-component/) wraps all of this in
  `<free-pdf-viewer>`.
