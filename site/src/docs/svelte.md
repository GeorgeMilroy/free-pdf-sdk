---
title: Svelte
description: Embed the viewer in Svelte and SvelteKit with the freePdf action, reactive options, fps-* events and the onViewer callback.
section: Frameworks
order: 44
---

# Svelte

`free-pdf-sdk/svelte` exports `freePdf`, a Svelte action for Svelte 4 and 5. Put it on an
element with a height; the viewer fills that element. Changed options call the matching API
methods, the element receives `fps-*` DOM events, and `onViewer` hands you the full API.

```svelte
<script>
  import { freePdf } from "free-pdf-sdk/svelte";
</script>

<div
  style="height: 80vh"
  use:freePdf={{ baseUrl: "/free-pdf-sdk/", document: { url: "/files/contract.pdf" } }}
></div>
```

## Install

```bash
npm install free-pdf-sdk
npx free-pdf-sdk copy-viewer static/free-pdf-sdk
```

The second command copies the viewer files (`viewer/`, `version.json`, the licences) into a
folder your app serves. SvelteKit serves `static/` at the site root; a plain Vite + Svelte project
serves `public/` (use `public/free-pdf-sdk` there). Either way the viewer is at `/free-pdf-sdk/`.
That URL is `baseUrl`, and it must be on the same origin as the app.

Run the command again after every upgrade, for example from a `postinstall` script:

```json
{
  "scripts": {
    "postinstall": "free-pdf-sdk copy-viewer static/free-pdf-sdk"
  }
}
```

## Options

The action takes every `FreePdf.init` option except `listeners`, plus `onViewer`:

| Option | Type | Notes |
|---|---|---|
| `baseUrl` | `string` | Required. URL of the viewer folder, on the page's origin. |
| `document` | `{ url, name?, fetchInit? }` or `{ data, name? }` | The PDF to open. `data` is an `ArrayBuffer`, `Uint8Array` or `Blob`. |
| `mode` | `Mode` | `view` (default, read-only), `review`, `annotate`, `fill`, `sign`, `organize`, `edit` or `full`. |
| `features` | `FeatureFlags` | Feature switches on top of the mode, such as `{ stamps: false }`. |
| `ui` | `UiOptions` | Interface options, such as `{ toolsPanel: false }`. |
| `disabledElements` | `ElementId[]` | Parts of the interface to hide, such as `["download", "print"]`. |
| `theme` | `"light" \| "dark" \| "system"` | Default `system`. |
| `author` | `string` | Author of new comments. |
| `locale` | `string` | Interface language, such as `en`, `de` or `zh-Hans`. |
| `password` | `string` | Used when `document` is opened: opens a protected file and lifts owner restrictions. |
| `timeoutMs` | `number` | How long requests wait for a reply (default 60000). Read once, at mount. |
| `onViewer` | `(viewer) => void` | Called with the viewer's API once it is ready and its first document has loaded or failed to (and again after a new `baseUrl` mounts a new viewer). |

The type is `FreePdfActionOptions`, exported from `free-pdf-sdk/svelte`.

The viewer is an iframe that fills the element. Give the element a height (and leave it empty:
the iframe is added inside it).

## Reactive options

When the options object changes, the action's `update` turns what changed into API calls, in
this order: `setMode`, `setFeatures`, `setUi`, `setDisabledElements`, `setTheme`, `setAuthor`,
`setLocale`, then `load` for a new `document`. A refused change is reported as an `fps-error`
event. Changing `baseUrl` destroys the viewer and mounts a new one.

Svelte 4:

```svelte
<script>
  import { freePdf } from "free-pdf-sdk/svelte";

  let mode = "review";
  let dark = false;
  let stamps = false;
</script>

<select bind:value={mode}>
  <option value="view">View</option>
  <option value="review">Review</option>
  <option value="annotate">Annotate</option>
</select>
<label><input type="checkbox" bind:checked={dark} /> Dark</label>
<label><input type="checkbox" bind:checked={stamps} /> Stamps</label>

<div
  class="viewer"
  use:freePdf={{
    baseUrl: "/free-pdf-sdk/",
    mode,
    theme: dark ? "dark" : "light",
    features: { stamps },
    document: { url: "/files/contract.pdf" },
  }}
></div>

<style>
  .viewer { height: calc(100vh - 64px); }
</style>
```

Svelte 5 with runes works the same way:

```svelte
<script lang="ts">
  import type { Mode } from "free-pdf-sdk";
  import { freePdf } from "free-pdf-sdk/svelte";

  let mode = $state<Mode>("review");
  let dark = $state(false);
</script>

<div
  style="height: 80vh"
  use:freePdf={{ baseUrl: "/free-pdf-sdk/", mode, theme: dark ? "dark" : "light", document: { url: "/files/contract.pdf" } }}
></div>
```

How changes are detected:

- `features`, `ui` and `disabledElements` are compared by value, so a new object with the same
  content causes no call.
- `document` with a `url` is compared by `url` and `name`; with `data`, by the identity of the
  bytes. Keep the same `ArrayBuffer`, `Uint8Array` or `Blob` to avoid reloads.
- `features` replaces the overrides: leaving it out goes back to the mode's own features. `ui`
  options you leave out keep their value, and leaving out `mode`, `theme`, `author` or `locale`
  keeps the current value.
- `password` is only used when a document is opened; change it together with `document`.

## Events

The element receives a DOM `CustomEvent` for every viewer event, with the data in `detail`.
These events don't bubble, so listen on the element that has the action.

| Event | `detail` |
|---|---|
| `fps-ready` | `{ sdk, pdfcraftRev }` |
| `fps-document-loaded` | `{ pageCount, permissions }` |
| `fps-document-error` | `{ code, message }` |
| `fps-annotations-changed` | `{ count }` |
| `fps-dirty-changed` | `{ dirty }` |
| `fps-error` | `{ code, message }` |
| `fps-save-requested` | `{ name, saveAs, data }` (`data` is the saved PDF as a `Uint8Array`) |

```svelte
<script>
  import { freePdf } from "free-pdf-sdk/svelte";
  let count = 0;
  let dirty = false;
</script>

<p>{count} comments{dirty ? " (unsaved)" : ""}</p>

<div
  style="height: 80vh"
  use:freePdf={{ baseUrl: "/free-pdf-sdk/", mode: "annotate", document: { url: "/files/contract.pdf" } }}
  on:fps-annotations-changed={(e) => (count = e.detail.count)}
  on:fps-dirty-changed={(e) => (dirty = e.detail.dirty)}
  on:fps-error={(e) => console.error(e.detail.code, e.detail.message)}
></div>
```

The `on:` directive works in Svelte 4 and 5. In Svelte 5 you can use event attributes instead,
for example `onfps-annotations-changed={(e) => (count = e.detail.count)}`.

## Get the API with `onViewer`

`onViewer` is called once, with the viewer's API, when the viewer is ready and the first
document is open. Keep the API in a variable and call it from your handlers. Only the
`onViewer` of the initial options is used.

```svelte
<script lang="ts">
  import type { FreePdfViewer } from "free-pdf-sdk";
  import { freePdf } from "free-pdf-sdk/svelte";

  export let contractId: string;
  export let userName: string;

  let viewer: FreePdfViewer | null = null;
  let count = 0;
  let status = "";

  async function restore(v: FreePdfViewer) {
    viewer = v;
    const res = await fetch(`/api/contracts/${contractId}/annotations`);
    if (res.ok) await v.importXfdf(await res.text(), { replace: true });
  }

  async function save() {
    if (!viewer) return;
    const xfdf = await viewer.exportXfdf();
    await fetch(`/api/contracts/${contractId}/annotations`, {
      method: "PUT",
      headers: { "Content-Type": "application/vnd.adobe.xfdf" },
      body: xfdf,
    });
    status = "Saved";
  }

  async function upload(e: CustomEvent<{ data: Uint8Array }>) {
    await fetch(`/api/contracts/${contractId}/pdf`, { method: "PUT", body: e.detail.data });
    status = "PDF saved";
  }
</script>

<header>
  {count} comments
  <button on:click={save} disabled={!viewer}>Save comments</button>
  {status}
</header>

<div
  class="viewer"
  use:freePdf={{
    baseUrl: "/free-pdf-sdk/",
    mode: "annotate",
    author: userName,
    disabledElements: ["download"],
    document: { url: `/api/contracts/${contractId}/pdf` },
    onViewer: restore,
  }}
  on:fps-annotations-changed={(e) => (count = e.detail.count)}
  on:fps-save-requested={upload}
></div>

<style>
  .viewer { height: calc(100vh - 48px); }
</style>
```

When the element is removed (the component is destroyed, an `{#if}` turns false), the action
destroys the viewer and pending calls reject with `DESTROYED`. Set your `viewer` variable back
to `null` if the element can go away while the component stays.

## SvelteKit

Server rendering works as is. Svelte runs actions only in the browser, and `free-pdf-sdk/svelte`
touches no browser globals until the action runs, so you can import it in any component and keep
SSR on. On the server the element renders empty; the viewer mounts when the page hydrates.

```svelte
<!-- src/routes/contracts/[id]/+page.svelte -->
<script>
  import { page } from "$app/stores";
  import { freePdf } from "free-pdf-sdk/svelte";
</script>

<div
  style="height: calc(100vh - 64px)"
  use:freePdf={{ baseUrl: "/free-pdf-sdk/", mode: "review", document: { url: `/api/contracts/${$page.params.id}/pdf` } }}
></div>
```

If the app is served under a base path (`paths.base`), include it in `baseUrl`.

## Things to know

- The viewer is an iframe on your origin that talks to the page over `postMessage`. PDFs are
  processed in the browser with WebAssembly and are not uploaded anywhere by the SDK. It needs
  WebGL; if it can't start, `fps-error` reports `TIMEOUT` (usually a wrong `baseUrl`) or
  `INTERNAL` (for example without WebGL).
- If a document can't be opened, `fps-document-error` reports why and the viewer keeps running:
  pass another `document` to try again.
- Prefer a ready-made element? `<free-pdf-viewer>` works in Svelte too; see
  [Web Component](/docs/web-component/).
