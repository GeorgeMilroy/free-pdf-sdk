---
title: API reference
description: FreePdf.init, every FreePdfViewer method and event, error codes, types and limits of the Free PDF SDK JavaScript API.
section: Reference
order: 70
---

# API reference

```ts
import { FreePdf, FreePdfError } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/contract.pdf" },
  mode: "annotate",
  features: { stamps: false },
  author: "Priya Raman",
  theme: "system",
  listeners: {
    documentLoaded: ({ pageCount }) => console.log(`${pageCount} pages`),
  },
});

viewer.on("annotationsChanged", ({ count }) => console.log(`${count} comments`));
const xfdf = await viewer.exportXfdf();
```

The package `free-pdf-sdk` exports `FreePdf` (also the default export), `init`,
`FreePdfError`, the constants `SDK_VERSION`, `PROTOCOL_VERSION`, `MODES` and `FEATURES`, and
every type on this page. The TypeScript declarations are the authoritative reference; this page
describes them.

Each viewer is an iframe on your page's own origin. The wrapper talks to it over a versioned
`postMessage` protocol, so **every method is async** and rejects with a `FreePdfError`.

## `FreePdf.init(element, options)`

```ts
function init(element: HTMLElement, options: InitOptions): Promise<FreePdfViewer>;
```

Mounts an iframe that fills `element`, waits for the viewer to start, applies the options,
opens `options.document` if one is given, and resolves to a `FreePdfViewer`. Give `element` a
size: the viewer fills it, and an element with no height shows nothing.

If anything fails (the viewer doesn't start, the options are invalid, the document can't be
opened) the promise rejects with a `FreePdfError` and the iframe is removed.

| Rejects with | When |
|---|---|
| `BAD_REQUEST` | `element` is not an `HTMLElement`, `baseUrl` is missing, `baseUrl` is on another origin, or an option is invalid (unknown mode, feature, `ui` option, panel or element id; empty `author`...). |
| `TIMEOUT` | The viewer did not start within 30 seconds: usually a wrong `baseUrl` (the iframe loaded something other than the viewer). |
| `INTERNAL` | The viewer page loaded but could not start, for example because WebGL is unavailable. |
| `UNSUPPORTED_PROTOCOL` | The viewer files speak another protocol version than the wrapper. Serve the viewer files from the same release as the package. |
| any `load` error | `options.document` could not be opened (see [`load`](#load)). |

If the wrapper and viewer versions differ but speak the same protocol, the viewer still starts
and a warning is logged to the console. Run `npx free-pdf-sdk copy-viewer <folder>` after every
upgrade so they match.

### `InitOptions`

| Option | Type | Default | Description |
|---|---|---|---|
| `baseUrl` | `string` | required | URL of the SDK folder (the one holding `viewer/` and `version.json`). It **must** be on the page's origin. A trailing `/` is added if missing; relative URLs resolve against the page. |
| `document` | `DocumentSource` | none | A PDF to open once the viewer is ready. |
| `mode` | `Mode` | `"view"` | A preset of features. `view` is read-only. See [Configuration](/docs/configuration/). |
| `features` | `FeatureFlags` | `{}` | Turn individual features on or off, on top of the mode. |
| `ui` | `UiOptions` | all defaults | Show or hide parts of the interface. |
| `disabledElements` | `ElementId[]` | `[]` | Single commands and tools to hide. |
| `author` | `string` | the name saved in the viewer's preferences | Author of new comments (written to XFDF `title`). Wins over the saved name. |
| `theme` | `"light" \| "dark" \| "system"` | `"system"` | Colour theme; `system` follows the browser. |
| `password` | `string` | none | Opens a document that needs a password, and lifts owner restrictions (as `unlock`). Used with `document`. |
| `locale` | `string` | the browser's language | Interface language such as `en`, `de` or `zh-Hans`; the closest available is used, English otherwise. |
| `loader` | `string \| LoaderOptions \| false` | a built-in spinner | The loading animation shown while the viewer starts and while a document opens. A string is an image URL. `false` shows nothing. See [Loading animation](/docs/theming-and-localization/#loading-animation). |
| `timeoutMs` | `number` | `60000` | How long to wait for a reply before rejecting with `TIMEOUT`. Loads (including fetching a URL) and `save` get twice this. |
| `listeners` | `{ [event]: listener }` | none | Listeners attached before the viewer starts, so they also hear events fired while `init` is running (`ready`, the first `documentLoaded`). |

### `DocumentSource`

```ts
type DocumentSource =
  | { url: string; name?: string; fetchInit?: RequestInit }
  | { data: ArrayBuffer | Uint8Array | Blob; name?: string };
```

- `url` is fetched by the wrapper in your page, so the usual rules apply: same origin, or a
  server that sends CORS headers. `fetchInit` is passed to `fetch` (headers, credentials, an
  abort `signal`). `name` defaults to the URL's last path segment.
- `data` is copied, never detached: you can keep using your buffer. `name` defaults to the
  `File`'s name, else `document.pdf`.

The document is read in the browser and handed to the viewer in the iframe. Nothing is uploaded
anywhere by the SDK.

## `FreePdfViewer` methods

Every method returns a promise. Methods marked **needs a document** reject with `NO_DOCUMENT`
when no document is open, and with `BUSY` while a document is loading. Any method can also
reject with `TIMEOUT` (no reply within `timeoutMs`) or `DESTROYED` (the viewer was destroyed).

### Documents

#### `load`

```ts
load(document: DocumentSource, options?: { password?: string }): Promise<LoadResult>
```

Opens a document, replacing the one shown. Resolves to `{ pageCount, permissions, unlocked }`
once it is open, and fires `documentLoaded`.

`password` is tried in two ways: if the document needs a password to open, it is used to open
it; otherwise, if the document has owner restrictions, it is tried as the owner password.
`unlocked` is `true` when it lifted those restrictions. A wrong owner password does not fail
the load: the document opens restricted, `unlocked` is `false` and an `error` event with
`WRONG_PASSWORD` fires.

| Rejects with | When |
|---|---|
| `FETCH_FAILED` | A `url` could not be fetched (network error, HTTP error status, or no response within twice `timeoutMs`). |
| `NEEDS_PASSWORD` | The document needs a password to open and none was given. |
| `WRONG_PASSWORD` | The password given does not open it. |
| `LOAD_FAILED` | The file is not a PDF the viewer can open. |
| `TOO_LARGE` | The PDF is larger than 512 MiB. |
| `BAD_REQUEST` | The PDF is empty, or the source has neither `url` nor `data`. |
| `BUSY` | Another document is still loading. |

A failed fetch (`FETCH_FAILED`, or `BAD_REQUEST` for a source with neither `url` nor `data`) and
a file the viewer can't open (`LOAD_FAILED`, `NEEDS_PASSWORD`, `WRONG_PASSWORD`) also fire
`documentError`. When the viewer itself can't open the file, the document shown before is
already closed, so no document is open afterwards; a failed fetch leaves it open.

```ts
try {
  await viewer.load({ url: "/files/protected.pdf" });
} catch (e) {
  if (e instanceof FreePdfError && e.code === "NEEDS_PASSWORD") {
    await viewer.load({ url: "/files/protected.pdf" }, { password: prompt("Password") ?? "" });
  } else {
    throw e;
  }
}
```

#### `unlock`

```ts
unlock(password: string): Promise<Permissions>
```

Lifts the open document's owner restrictions with its owner password and resolves to the new
permissions. It changes nothing in the file: a PDF saved afterwards keeps its protection. If
the document has no restrictions to lift, it resolves to the current permissions.
**Needs a document.** Rejects with `WRONG_PASSWORD`.

#### `getPermissions`

```ts
getPermissions(): Promise<Permissions>
```

What the open document's security allows. **Needs a document.**

#### `getPageCount`

```ts
getPageCount(): Promise<number>
```

The number of pages. **Needs a document.**

#### `isDirty`

```ts
isDirty(): Promise<boolean>
```

Whether the document has unsaved changes. Resolves to `false` when no document is open.

#### `save`

```ts
save(): Promise<Uint8Array>
```

The PDF with every change, comments included. The viewer then treats the document as saved:
`isDirty()` becomes `false` and `dirtyChanged` fires. It works in every mode, whether or not the
`save` feature (the viewer's own Save button) is on. **Needs a document.**

```ts
const pdf = await viewer.save();
await fetch("/api/files/contract.pdf", { method: "PUT", body: pdf });
```

### Annotations

See [Annotations and XFDF](/docs/annotations-and-xfdf/) for a full guide.

#### `importXfdf`

```ts
importXfdf(xfdf: string | Blob, options?: { replace?: boolean }): Promise<number>
```

Imports comments from standard XFDF, whichever PDF tool wrote it. With `replace: true` the existing comments are
deleted first; nothing is deleted when the XFDF is invalid. Resolves to the number of top-level
comments imported (replies are not counted; comments on pages the document doesn't have are
skipped). An XFDF with no comments imports 0. **Needs a document.**

| Rejects with | When |
|---|---|
| `READ_ONLY` | No annotation feature is on (for example `view` or `organize` mode), or the document can't be edited. |
| `NOT_PERMITTED` | The document's security doesn't allow comments; unlock it with the owner password. |
| `XFDF_INVALID` | The XFDF can't be read, or it holds form field values (it imports comments only). |
| `TOO_LARGE` | The XFDF is larger than 32 MiB. |

#### `exportXfdf`

```ts
exportXfdf(): Promise<string>
```

The document's comments as XFDF, with the author in each comment's `title` attribute.
**Needs a document.**

#### `getAnnotations`

```ts
getAnnotations(): Promise<Annotation[]>
```

The document's top-level comments, each with its replies. The array's length is the exact
number of comments: it counts what `exportXfdf()` writes. **Needs a document.**

#### `deleteAllAnnotations`

```ts
deleteAllAnnotations(): Promise<number>
```

Deletes every comment, locked ones included, and resolves to how many there were.
**Needs a document.** Rejects with `READ_ONLY` (no annotation feature is on, or the document
can't be edited) or `NOT_PERMITTED` (the document's security doesn't allow comments).

#### `setAuthor`

```ts
setAuthor(author: string): Promise<void>
```

Sets the author of new comments. The name is trimmed and control characters are removed; it
stays in effect for documents loaded later. Rejects with `BAD_REQUEST` when empty and
`TOO_LARGE` over 200 characters.

### Configuration

See [Configuration](/docs/configuration/) for what each option does.

#### `setMode`

```ts
setMode(mode: Mode): Promise<void>
```

Switches mode. Your feature overrides still apply on top of it. Rejects with `BAD_REQUEST` for
an unknown mode.

#### `setFeatures`

```ts
setFeatures(features: FeatureFlags): Promise<void>
```

Replaces the feature overrides (relative to the mode). Pass `{}` to use the mode's features as
they are. Rejects with `BAD_REQUEST` for an unknown feature or a value that isn't a boolean.

#### `setUi`

```ts
setUi(ui: UiOptions): Promise<void>
```

Changes interface options; options you leave out keep their value, and panels you don't name
keep theirs. Rejects with `BAD_REQUEST` for an unknown option or panel.

#### `setDisabledElements`

```ts
setDisabledElements(ids: ElementId[]): Promise<void>
```

Hides parts of the interface, replacing the previous list. Rejects with `BAD_REQUEST` for an
unknown id and `TOO_LARGE` for more than 256 ids.

#### `getConfig`

```ts
getConfig(): Promise<ViewerConfig>
```

The configuration in effect: the mode, every feature on or off, the interface options, the
hidden elements and whether the viewer is read-only.

### Display

#### `setTheme`

```ts
setTheme(theme: "light" | "dark" | "system"): Promise<void>
```

Sets the colour theme. Rejects with `BAD_REQUEST` for any other value.

#### `setLocale`

```ts
setLocale(locale: string): Promise<string>
```

Sets the interface language and resolves to the language actually used: the exact tag if
available, else its primary language (`de` for `de-AT`), else `en`. Rejects with
`BAD_REQUEST` if `locale` is not a language tag (letters, digits, `-` and `_`).

#### `goToPage`

```ts
goToPage(page: number): Promise<void>
```

Shows page `page`, counted from **1**. **Needs a document.** Rejects with `BAD_REQUEST` when
`page` is not a whole number from 1 to the page count.

#### `setZoom`

```ts
setZoom(zoom: number | "fit-width" | "fit-page"): Promise<void>
```

Sets the zoom: a percentage from 10 to 6400, or a fit. **Needs a document.** Rejects with
`BAD_REQUEST` outside that range.

### Lifecycle

#### `version`

```ts
version(): Promise<VersionInfo>
```

Resolves to `{ sdk, pdfcraftRev }`: the SDK version of the viewer files and the engine
revision they were built from. `FreePdf.version` is the wrapper's own version, available
without a viewer.

#### `destroy`

```ts
destroy(): Promise<void>
```

Removes the iframe and every listener. Pending requests reject with `DESTROYED`, and so does
every later call. Safe to call twice. Call it when your component unmounts.

#### `on` / `off`

```ts
on<K extends EventName>(name: K, listener: (data: EventMap[K]) => void): void
off<K extends EventName>(name: K, listener: (data: EventMap[K]) => void): void
```

Adds or removes an event listener. A listener that throws is logged and does not stop the
others. A `ready` listener added after the viewer is ready is still called once.

#### `iframe`

```ts
readonly iframe: HTMLIFrameElement
```

The iframe holding the viewer, for example to focus it.

## Events

| Event | Data | When |
|---|---|---|
| `ready` | `{ sdk: string; pdfcraftRev: string }` | The viewer is configured and ready for requests. Fires before the first document opens. |
| `documentLoaded` | `{ pageCount: number; permissions: Permissions }` | A document opened. |
| `documentError` | `{ code: ErrorCode; message: string }` | A document could not be opened (including a failed fetch). |
| `annotationsChanged` | `{ count: number }` | Comments were added, changed or removed, by the user or the API. `count` is the number of top-level comments. |
| `dirtyChanged` | `{ dirty: boolean }` | The document gained or lost unsaved changes. |
| `error` | `{ code: ErrorCode; message: string }` | Something failed outside a request, for example a wrong owner `password` at load (the document opens restricted). |
| `saveRequested` | `{ name: string; saveAs: boolean; data: Uint8Array }` | The user pressed Save (or Save As) in the viewer. `data` is the saved PDF; the viewer treats the document as saved. Store it yourself. |

```ts
viewer.on("dirtyChanged", ({ dirty }) => {
  saveButton.disabled = !dirty;
});

viewer.on("saveRequested", async ({ name, data }) => {
  await fetch(`/api/files/${encodeURIComponent(name)}`, { method: "PUT", body: data });
});
```

`ready` and the first `documentLoaded` fire while `init` is still running. To hear them, pass
the listeners in `options.listeners` (a `ready` listener added later with `on` is still called).

## Errors

Every failure is a `FreePdfError`, a subclass of `Error` with a machine-readable `code`:

```ts
class FreePdfError extends Error {
  readonly code: ErrorCode;
}
```

```ts
try {
  await viewer.importXfdf(xfdf, { replace: true });
} catch (e) {
  if (!(e instanceof FreePdfError)) throw e;
  switch (e.code) {
    case "NOT_PERMITTED":
      showMessage("This file is protected. Unlock it to add comments.");
      break;
    case "XFDF_INVALID":
      showMessage("The stored comments could not be read.");
      break;
    default:
      console.error(e.code, e.message);
  }
}
```

Messages are in English and meant for logs, not for users: switch on `code`.

| Code | Meaning |
|---|---|
| `BAD_REQUEST` | A parameter is missing, mistyped or out of range, or a name (mode, feature, `ui` option, panel, element id) is unknown. |
| `UNKNOWN_METHOD` | The viewer has no such method (the wrapper and viewer are from different releases). |
| `UNSUPPORTED_PROTOCOL` | The viewer speaks another protocol version than the wrapper. |
| `NOT_READY` | The viewer's iframe is not attached to the page. |
| `NO_DOCUMENT` | No document is open. |
| `LOAD_FAILED` | The document could not be opened. |
| `NEEDS_PASSWORD` | The document needs a password to open. |
| `WRONG_PASSWORD` | The password is wrong. |
| `NOT_PERMITTED` | The document's security settings don't allow this; unlock it with the owner password. |
| `READ_ONLY` | No annotation feature is on (for example in `view` mode), or the document can't be edited. |
| `XFDF_INVALID` | The XFDF could not be read, or it holds form field values. |
| `TOO_LARGE` | A value is larger than the protocol allows (see [Limits](#limits)). |
| `BUSY` | A document is loading, or the request expired before the viewer could run it. |
| `INTERNAL` | Something failed inside the viewer, or the viewer could not start (for example without WebGL). |
| `TIMEOUT` | From the wrapper: no reply within `timeoutMs`, or the viewer did not start within 30 seconds. |
| `DESTROYED` | From the wrapper: the viewer was destroyed. |
| `FETCH_FAILED` | From the wrapper: a document URL could not be fetched. |

A request that times out in the wrapper never takes effect later: every request carries a
deadline, and the viewer drops it (with `BUSY`) if it reaches the viewer too late, as can
happen in a background tab.

## Types

### `Mode`, `Feature`, `FeatureFlags`

```ts
type Mode = "view" | "review" | "annotate" | "fill" | "sign" | "organize" | "edit" | "full";

type Feature =
  | "comments" | "markup" | "shapes" | "ink" | "stamps" | "measurement"
  | "fillSign" | "formFill" | "formDesign" | "digitalSignatures" | "signaturePanel"
  | "pages" | "bookmarks" | "editContent" | "flatten" | "redaction"
  | "accessibility" | "standards" | "javascript" | "security" | "history"
  | "save" | "download" | "export" | "print" | "search" | "copy"
  | "documentInfo" | "help" | "palette" | "preferences";

type FeatureFlags = Partial<Record<Feature, boolean>>;
```

`MODES` and `FEATURES` export the same lists as arrays. What each mode and feature means is in
[Configuration](/docs/configuration/).

### `UiOptions` and `PanelName`

```ts
interface UiOptions {
  toolsPanel?: boolean;       // default true
  toolsRail?: boolean;        // default true
  quickToolbar?: boolean;     // default true
  rightRail?: boolean;        // default true
  zoomControls?: boolean;     // default true
  pageNavigation?: boolean;   // default true
  menu?: boolean;             // default true
  searchBox?: boolean;        // default true (needs the palette feature)
  panels?: Partial<Record<PanelName, boolean>>;
  lockLayout?: boolean;       // default false
  showPlannedTools?: boolean; // default false
  externalLinks?: boolean;    // default true
}

type PanelName =
  | "comments" | "bookmarks" | "pages" | "fields" | "layers"
  | "attachments" | "signatures" | "accessibility" | "tags" | "search";
```

### `ElementId`

The ids `disabledElements` accepts: `download`, `save`, `print`, `open`, `flatten`,
`settings`, `import`, and the tools `tool:note`, `tool:freetext`, `tool:callout`,
`tool:highlight`, `tool:underline`, `tool:strikeout`, `tool:squiggly`, `tool:caret`,
`tool:replace`, `tool:ink`, `tool:line`, `tool:arrow`, `tool:square`, `tool:circle`,
`tool:polygon`, `tool:polyline`, `tool:cloud`, `tool:attach`, `tool:stamp`, `tool:image`,
`tool:link`, `tool:measure`, `tool:fillsign`, `tool:signature`. What each one hides is in
[Configuration](/docs/configuration/#hide-single-commands-disabledelements).

### `ViewerConfig`

What `getConfig()` resolves to.

```ts
interface ViewerConfig {
  mode: Mode;
  features: Record<Feature, boolean>; // every feature: the mode's preset with your overrides
  ui: Required<Omit<UiOptions, "panels">> & { panels: Partial<Record<PanelName, boolean>> };
  disabledElements: ElementId[];
  readOnly: boolean;                  // no feature that changes the document is on
}
```

### `Permissions`

What the open document's security allows.

```ts
interface Permissions {
  canAnnotate: boolean; // comments may be added, changed, deleted and imported
  canPrint: boolean;
  canCopy: boolean;
  isEncrypted: boolean;
  canUnlock: boolean;   // an owner password would lift restrictions (unlock)
}
```

`canAnnotate` is `true` only when the document allows both adding comments and modifying it,
because importing comments needs both. A file protected with an owner password that forbids
commenting reports `false` until you call `unlock(ownerPassword)`.

### `Annotation`

What `getAnnotations()` lists.

```ts
interface Annotation {
  id: string;              // XFDF name
  type: string;            // text (note), freetext, square, circle, line, ink, polygon,
                           // polyline, highlight, underline, squiggly, strikeout, stamp,
                           // caret, fileattachment, sound, redact
  author: string | null;   // XFDF title
  subject: string | null;
  contents: string | null; // plain text
  date: string | null;     // ISO 8601: last modified, else created
  pageIndex: number;       // 0-based
  color: string | null;    // "#RRGGBB"
  replies: AnnotationReply[]; // oldest first
}

interface AnnotationReply {
  id: string;
  author: string | null;
  contents: string | null;
  date: string | null;
}
```

`type` is the PDF annotation subtype in lowercase.

### Other types

```ts
interface DocumentInfo { pageCount: number; permissions: Permissions } // documentLoaded
interface LoadResult extends DocumentInfo { unlocked: boolean }        // load()
interface VersionInfo { sdk: string; pdfcraftRev: string }             // ready, version()
interface ErrorInfo { code: ErrorCode; message: string }               // documentError, error
type Theme = "light" | "dark" | "system";
type Zoom = number | "fit-width" | "fit-page";
type EventName = keyof EventMap;
type Listener<K extends EventName> = (data: EventMap[K]) => void;
```

## Limits

| Value | Limit |
|---|---|
| PDF size | 512 MiB (`TOO_LARGE`) |
| XFDF size | 32 MiB (`TOO_LARGE`) |
| Document name | 512 characters; path separators and control characters are removed |
| Author | 200 characters, not empty |
| Password | 1024 characters |
| Locale tag | 35 characters |
| `disabledElements` | 256 ids |
| Zoom | 10 to 6400 percent |
| Page number | from 1 to the page count |
| Viewer start | 30 seconds (`TIMEOUT`) |
| Request reply | `timeoutMs` (default 60 seconds); loads and `save` twice that |

Only one document is open per viewer, and one load runs at a time. To show two documents at
once, mount two viewers.

## Requirements

- The SDK folder (`baseUrl`) is served from the page's own origin. The viewer runs in an
  iframe on that origin and only talks to its parent there.
- The browser supports WebAssembly and WebGL.
- Documents are processed in the browser; the SDK has no server component and uploads nothing.
