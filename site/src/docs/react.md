---
title: React
description: Embed the viewer with the FreePdfViewer React component, its props, event handlers and ref handle, including Next.js App Router setup.
section: Frameworks
order: 41
---

# React

`free-pdf-sdk/react` exports `FreePdfViewer`, a component for React 17, 18 and 19. Props
configure the viewer, `on*` props receive its events, and a ref gives you the full API.

```tsx
import { FreePdfViewer } from "free-pdf-sdk/react";

export function ContractViewer() {
  return (
    <FreePdfViewer
      baseUrl="/free-pdf-sdk/"
      document={{ url: "/files/contract.pdf" }}
      style={{ height: "80vh" }}
    />
  );
}
```

## Install

```bash
npm install free-pdf-sdk
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies the viewer files (`viewer/`, `version.json`, the licences) into
`public/free-pdf-sdk`, which Vite, Create React App and Next.js serve at `/free-pdf-sdk/`. That URL
is `baseUrl`, and it must be on the same origin as your app. Run the command again after every
upgrade, for example from a script:

```json
{
  "scripts": {
    "postinstall": "free-pdf-sdk copy-viewer public/free-pdf-sdk"
  }
}
```

## Props

The component takes every `FreePdf.init` option as a prop, plus event handlers and the styling
of its container.

| Prop | Type | Notes |
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
| `className` | `string` | Class of the container `div` the viewer fills. |
| `style` | `CSSProperties` | Style of that `div`. Give it a height. |

The container has `position: relative` and a `min-height` of 200px. Without a height it stays
200px tall: set one through `style` or `className` (a fixed height, `100%` of a sized parent,
`flex: 1` in a flex column).

## Events

Each viewer event has a handler prop:

| Prop | Receives | When |
|---|---|---|
| `onReady` | `{ sdk, pdfcraftRev }` | The viewer is configured and ready for requests. |
| `onDocumentLoaded` | `{ pageCount, permissions }` | A document opened. |
| `onDocumentError` | `{ code, message }` | A document could not be opened. |
| `onAnnotationsChanged` | `{ count }` | Comments were added, changed or removed, by the user or the API. |
| `onDirtyChanged` | `{ dirty }` | The document gained or lost unsaved changes. |
| `onError` | `{ code, message }` | Something failed outside a request: the viewer could not start, a prop change was refused. |
| `onSaveRequested` | `{ name, saveAs, data }` | The user pressed Save in the viewer. `data` is the PDF as a `Uint8Array`; the viewer now treats it as saved. |

The component always calls the handler from the latest render, so inline arrow functions are
fine and you don't need `useCallback`.

```tsx
const [count, setCount] = useState(0);
const [dirty, setDirty] = useState(false);

<FreePdfViewer
  baseUrl="/free-pdf-sdk/"
  mode="annotate"
  document={{ url: "/files/contract.pdf" }}
  onAnnotationsChanged={({ count }) => setCount(count)}
  onDirtyChanged={({ dirty }) => setDirty(dirty)}
  onError={({ code, message }) => console.error(code, message)}
  style={{ height: "80vh" }}
/>;
```

## Call the API with a ref

A ref to the component is a `FreePdfViewerHandle` with two methods:

- `viewer()` returns the viewer's API once it is ready, else `null`.
- `whenReady()` resolves with the API once the viewer is ready and its first document has loaded (or failed to).

```tsx
import { useRef } from "react";
import { FreePdfViewer, type FreePdfViewerHandle } from "free-pdf-sdk/react";

export function Viewer() {
  const ref = useRef<FreePdfViewerHandle>(null);

  async function goToLastPage() {
    const viewer = await ref.current?.whenReady();
    if (!viewer) return;
    await viewer.goToPage(await viewer.getPageCount());
  }

  return (
    <>
      <button onClick={goToLastPage}>Last page</button>
      <FreePdfViewer ref={ref} baseUrl="/free-pdf-sdk/" document={{ url: "/files/report.pdf" }} style={{ height: "80vh" }} />
    </>
  );
}
```

Every API method is async and rejects with a `FreePdfError` that has a `code` and a `message`.
Import the class and the types from the main entry point:

```ts
import { FreePdfError, type FreePdfViewer as Api } from "free-pdf-sdk";
```

## Change props

Changing a prop calls the matching API method on the running viewer, in this order: `setMode`,
`setFeatures`, `setUi`, `setDisabledElements`, `setTheme`, `setAuthor`, `setLocale`, then `load`
for a new `document`. A refused change is reported through `onError`.

```tsx
import type { Mode } from "free-pdf-sdk";

const [mode, setMode] = useState<Mode>("view");
const [dark, setDark] = useState(false);

<FreePdfViewer
  baseUrl="/free-pdf-sdk/"
  mode={mode}
  theme={dark ? "dark" : "light"}
  features={{ stamps: false }}
  document={{ url: `/files/${fileId}.pdf` }}
  style={{ height: "80vh" }}
/>;
```

How changes are detected:

- `features`, `ui` and `disabledElements` are compared by value, so object literals written
  inline don't cause calls on every render.
- `document` with a `url` is compared by `url` and `name`. With `data`, it is compared by the
  identity of the bytes: keep the same `ArrayBuffer`, `Uint8Array` or `Blob` (in state, or with
  `useMemo`) or the document reloads on every render.
- `features` replaces the overrides: removing the prop goes back to the mode's own features.
  `ui` options you leave out keep their value, and removing `mode`, `theme`, `author` or `locale`
  keeps the current value.
- `password` is only used when a document is opened; change it together with `document`.
- Changing `baseUrl` destroys the viewer and mounts a new one. `timeoutMs` is read at mount.

Calls you make through the ref's API don't change the props. If you call `viewer.setMode()`
yourself, keep your `mode` prop in step, or the next prop change won't do what you expect.

## Example: annotate a contract and save the comments

This component opens a PDF in `annotate` mode, restores comments saved earlier as XFDF, and
saves them back to your server. It also handles the viewer's own Save button.

```tsx
import { useRef, useState } from "react";
import { FreePdfError } from "free-pdf-sdk";
import { FreePdfViewer, type FreePdfViewerHandle } from "free-pdf-sdk/react";

type Props = { contractId: string; userName: string };

export function ContractReview({ contractId, userName }: Props) {
  const ref = useRef<FreePdfViewerHandle>(null);
  const [count, setCount] = useState(0);
  const [status, setStatus] = useState("");

  // Runs for every document that opens: restore the comments saved for it.
  async function restoreComments() {
    const viewer = await ref.current?.whenReady();
    if (!viewer) return;
    const response = await fetch(`/api/contracts/${contractId}/annotations`);
    if (response.status === 404) return; // nothing saved yet
    const imported = await viewer.importXfdf(await response.text(), { replace: true });
    setStatus(`Restored ${imported} comments`);
  }

  async function saveComments() {
    const viewer = ref.current?.viewer();
    if (!viewer) return;
    try {
      const xfdf = await viewer.exportXfdf();
      await fetch(`/api/contracts/${contractId}/annotations`, {
        method: "PUT",
        headers: { "Content-Type": "application/vnd.adobe.xfdf" },
        body: xfdf,
      });
      setStatus("Comments saved");
    } catch (e) {
      setStatus(e instanceof FreePdfError ? `Could not save (${e.code})` : "Could not save");
    }
  }

  // The user pressed Save in the viewer: upload the whole PDF.
  async function uploadPdf(data: Uint8Array) {
    await fetch(`/api/contracts/${contractId}/pdf`, {
      method: "PUT",
      headers: { "Content-Type": "application/pdf" },
      body: data,
    });
    setStatus("PDF saved");
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <header style={{ display: "flex", gap: 12, alignItems: "center", padding: "8px 16px" }}>
        <span>{count} comments</span>
        <button onClick={saveComments}>Save comments</button>
        <span>{status}</span>
      </header>
      <FreePdfViewer
        ref={ref}
        baseUrl="/free-pdf-sdk/"
        mode="annotate"
        features={{ measurement: false }}
        disabledElements={["download"]}
        author={userName}
        document={{ url: `/api/contracts/${contractId}/pdf` }}
        onDocumentLoaded={() => void restoreComments()}
        onAnnotationsChanged={({ count }) => setCount(count)}
        onSaveRequested={({ data }) => void uploadPdf(data)}
        onError={({ code, message }) => setStatus(`${code}: ${message}`)}
        style={{ flex: 1 }}
      />
    </div>
  );
}
```

Notes on the example:

- `onDocumentLoaded` can fire before the ref's `viewer()` is set for the first document, so
  `restoreComments` waits with `whenReady()`.
- `annotate` mode includes comments, markup, shapes, ink, stamps, measurement, Fill & Sign,
  undo history and Save, and nothing that edits pages or content. `importXfdf` and
  `deleteAllAnnotations` need an annotation feature: without one (`view` or `organize` mode,
  say) they reject with `READ_ONLY`. See [Annotations and XFDF](/docs/annotations-and-xfdf/).
- XFDF carries the comments only. To keep form fields, flattened content or page changes, save
  the PDF itself with `viewer.save()` (or from `onSaveRequested`).

## Next.js (App Router)

The viewer runs only in the browser, so render it from a Client Component. Put the viewer folder
in `public/`:

```bash
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

Mark the component that uses `FreePdfViewer` with `"use client"`:

```tsx
// app/contracts/[id]/ContractViewer.tsx
"use client";

import { FreePdfViewer } from "free-pdf-sdk/react";

export default function ContractViewer({ id }: { id: string }) {
  return (
    <FreePdfViewer
      baseUrl="/free-pdf-sdk/"
      mode="review"
      document={{ url: `/api/contracts/${id}/pdf` }}
      style={{ height: "calc(100vh - 64px)" }}
    />
  );
}
```

```tsx
// app/contracts/[id]/page.tsx (a Server Component)
import ContractViewer from "./ContractViewer";

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <ContractViewer id={id} />;
}
```

On the server the component renders its empty container; the viewer mounts in the browser.

To keep the component out of the server render entirely (and load its code only when the page
needs it), import it with `next/dynamic` and `ssr: false`. In the App Router, `ssr: false` is only
allowed inside a Client Component, so wrap it:

```tsx
// app/contracts/[id]/LazyContractViewer.tsx
"use client";

import dynamic from "next/dynamic";

const ContractViewer = dynamic(() => import("./ContractViewer"), {
  ssr: false,
  loading: () => <p>Loading viewer…</p>,
});

export default ContractViewer;
```

Then render `<LazyContractViewer id={id} />` from the page. If your app uses a `basePath`, include
it in `baseUrl` (for example `/docs-app/free-pdf-sdk/`).

## StrictMode

In development, `<StrictMode>` mounts components, unmounts them and mounts them again. The
component handles this: the first viewer is destroyed before it is ready and its events are
dropped, so your handlers only hear from the second one. You may briefly see two iframes in the
development tools. Production builds mount once.

## Things to know

- Unmounting the component destroys the viewer; pending calls reject with `DESTROYED`.
- If a document can't be opened, `onDocumentError` fires and the viewer keeps running: set
  another `document` (or a `password` with the same document) to try again.
- The viewer is an iframe on your origin that talks to the page over `postMessage`. PDFs are
  processed in the browser with WebAssembly and are not uploaded anywhere by the SDK. It needs
  WebGL; if it can't start, `onError` reports `TIMEOUT` (usually a wrong `baseUrl`) or
  `INTERNAL` (for example without WebGL).
- The component is a thin layer over the core API; see [Vanilla JavaScript](/docs/vanilla-js/)
  for `FreePdf.init` itself.
