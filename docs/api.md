# API reference (v1)

```ts
import { FreePdf, FreePdfError } from "free-pdf-sdk";

const viewer = await FreePdf.init(element, {
  baseUrl: "/free-pdf-sdk/0.1.0/",   // the SDK folder, on this page's origin
  document: { url: pdfUrl },         // or { data: ArrayBuffer | Uint8Array | Blob, name }
  mode: "annotate",                  // "view" (default, read-only) | "annotate"
  author: "Priya Raman",
  theme: "system",                   // "light" | "dark" | "system"
  password: ownerPassword,           // optional: unlocks owner restrictions on load
  disabledElements: ["download", "print", "tool:stamp"],
  locale: "en",
  listeners: { documentLoaded: (info) => console.log(info.pageCount) },
});
```

`FreePdf.init(element, options)` mounts an iframe that fills `element` (give it a size), waits
for the viewer, applies the options, opens `options.document` if given, and resolves to a
`FreePdfViewer`. It rejects with a `FreePdfError` (and removes the iframe) if the viewer cannot
start or the document cannot be opened.

All methods are async and reject with `FreePdfError { code, message }`. The TypeScript types
(`free-pdf-sdk.d.ts`) are the reference; this page summarises them.

## Options

| Option | Type | Default | Notes |
|---|---|---|---|
| `baseUrl` | `string` | required | URL of the SDK folder (the one with `viewer/` and `version.json`). Must be on the page's origin. |
| `document` | `{ url, name?, fetchInit? }` or `{ data, name? }` | none | `url` is fetched by the wrapper (CORS applies); `data` is copied, never detached. |
| `mode` | `"view" \| "annotate"` | `"view"` | See [Modes](#modes). |
| `author` | `string` | the viewer's saved name, else `Guest` | Author (`title`) of new comments. Wins over a saved name. |
| `theme` | `"light" \| "dark" \| "system"` | `"system"` | |
| `password` | `string` | none | Opens a file that needs a password, and lifts owner restrictions (same as `unlock`). |
| `disabledElements` | `ElementId[]` | `[]` | See [Disabled elements](#disabled-elements). |
| `locale` | `string` | browser language | Closest interface language; English otherwise. |
| `timeoutMs` | `number` | `60000` | Per request; loads and saves get twice that. |
| `listeners` | `{ [event]: listener }` | none | Attached before start, so they hear `ready` and the first `documentLoaded`. |

## Methods

| Method | Resolves to | Errors |
|---|---|---|
| `load(document, { password? })` | `{ pageCount, permissions, unlocked }` | `FETCH_FAILED`, `LOAD_FAILED`, `NEEDS_PASSWORD`, `WRONG_PASSWORD`, `TOO_LARGE`, `BUSY` |
| `unlock(password)` | `Permissions` | `WRONG_PASSWORD`, `NO_DOCUMENT` |
| `getPermissions()` | `{ canAnnotate, canPrint, canCopy, isEncrypted, canUnlock }` | `NO_DOCUMENT` |
| `importXfdf(xfdf, { replace? })` | number imported | `READ_ONLY`, `NOT_PERMITTED`, `XFDF_INVALID`, `TOO_LARGE` |
| `exportXfdf()` | XFDF string (author in `title`) | `NO_DOCUMENT` |
| `getAnnotations()` | `Annotation[]` (exact count) | `NO_DOCUMENT` |
| `deleteAllAnnotations()` | number deleted | `READ_ONLY`, `NOT_PERMITTED` |
| `setAuthor(author)` | — | `BAD_REQUEST` (empty) |
| `setTheme(theme)` | — | `BAD_REQUEST` |
| `setMode(mode)` | — | `BAD_REQUEST` |
| `setDisabledElements(ids)` | — | `BAD_REQUEST` (unknown id) |
| `setLocale(locale)` | language used | `BAD_REQUEST` |
| `getPageCount()` | number | `NO_DOCUMENT` |
| `goToPage(page)` | — (1-based) | `BAD_REQUEST` (out of range) |
| `setZoom(zoom)` | — (percent 10–6400, `"fit-width"`, `"fit-page"`) | `BAD_REQUEST` |
| `isDirty()` | boolean | |
| `save()` | `Uint8Array` (the PDF with every change) | `NO_DOCUMENT` |
| `version()` | `{ sdk, pdfcraftRev }` | |
| `destroy()` | — | never; pending calls reject with `DESTROYED` |
| `on(name, listener)` / `off(name, listener)` | — | |

Notes:

* `permissions.canAnnotate` is true only when the document's security allows both adding
  comments and importing them (pdfcraft imports under the "modify" permission). For files
  protected with an owner password and no commenting, it is `false` until
  `unlock(ownerPassword)`.
* `importXfdf` returns how many of the XFDF's top-level comments are in the document afterwards
  (replies are not counted). An XFDF with no comments imports 0 without an error. With
  `replace: true` the existing comments are deleted first, in the same undo step.
* `save()` marks the document saved (`isDirty()` becomes false, `dirtyChanged` fires).
* `deleteAllAnnotations()` also deletes locked comments; one undo step restores them.

### `Annotation`

```ts
interface Annotation {
  id: string;              // XFDF name
  type: string;            // text (note), freetext, square, circle, line, ink, polygon, polyline,
                           // highlight, underline, squiggly, strikeout, stamp, caret,
                           // fileattachment, sound, redact
  author: string | null;   // XFDF title
  subject: string | null;
  contents: string | null; // plain text (rich text is flattened to text)
  date: string | null;     // ISO 8601; modified date, else creation date
  pageIndex: number;       // 0-based
  color: string | null;    // "#RRGGBB"
  replies: { id: string; author: string | null; contents: string | null; date: string | null }[];
}
```

The array's length is the exact number of comments. Review-status replies ("Accepted") are not
listed as replies.

## Events

| Event | Data | When |
|---|---|---|
| `ready` | `{ sdk, pdfcraftRev }` | The viewer is configured. Late listeners still get it. |
| `documentLoaded` | `{ pageCount, permissions }` | A document opened. |
| `documentError` | `{ code, message }` | A document could not be opened. |
| `annotationsChanged` | `{ count }` | Comments were added, changed or removed (UI or API). |
| `dirtyChanged` | `{ dirty }` | Unsaved changes appeared or went away. |
| `error` | `{ code, message }` | A failure outside a request (for example a wrong `password` at load: the document opens, restricted). |
| `saveRequested` | `{ name, saveAs, data: Uint8Array }` | The user pressed Save in the viewer. The viewer treats the document as saved. Disable `save` to remove the button. |

## Modes

* `view`: read-only. Nothing can change the document: the editing tools are hidden, edits,
  undo and save are refused inside the viewer, and `importXfdf`/`deleteAllAnnotations` reject
  with `READ_ONLY`.
* `annotate`: comments (notes, free text, callouts, highlight/underline/strikeout/squiggly,
  caret, replace), shapes (line, arrow, rectangle, oval, polygon, polyline, cloud), ink, replies
  and review status, Fill & Sign (typed and drawn signatures, initials, text, marks) and
  measurement (distance, perimeter, area). Page, content, form and security changes are hidden.

## Disabled elements

| Id | Hides (pdfcraft ids) |
|---|---|
| `download` | Save As, comment/data export, measurement export, Export a PDF (`file.save_as`, `comment.export`, `form.export_data`, `measure.export`, `export.*`, tool group `export`) |
| `save` | Save, Save As (`file.save`, `file.save_as`) |
| `print` | Print (`print.dialog`) |
| `open` | Open, Open Recent, Close, New blank PDF, Create and Combine tool groups, dropping files on the viewer |
| `flatten` | Flatten comments, Flatten form (`comment.flatten`, `form.flatten`) |
| `settings` | Preferences (`app.preferences`) |
| `import` | Import comments/data (`comment.import`, `form.import_data`); `importXfdf` still works |
| `tool:note`, `tool:freetext`, `tool:callout` | Note, Text box, Callout |
| `tool:highlight`, `tool:underline`, `tool:strikeout`, `tool:squiggly` | Text markup |
| `tool:caret`, `tool:replace` | Insert text, Replace text |
| `tool:ink` | Draw (and the ink eraser) |
| `tool:line`, `tool:arrow`, `tool:square`, `tool:circle`, `tool:polygon`, `tool:polyline`, `tool:cloud` | Shapes |
| `tool:attach` | File attachment |
| `tool:stamp` | Stamps (and the Add a stamp tool group) |
| `tool:image` | Add image |
| `tool:link` | Add link |
| `tool:measure` | Measurement tools (and the Measure tool group) |
| `tool:fillsign` | Fill & Sign (and its tool group) |
| `tool:signature` | Signature and initials only |

## Errors

`FreePdfError.code` is one of: `BAD_REQUEST`, `UNKNOWN_METHOD`, `UNSUPPORTED_PROTOCOL`,
`NOT_READY`, `NO_DOCUMENT`, `LOAD_FAILED`, `NEEDS_PASSWORD`, `WRONG_PASSWORD`, `NOT_PERMITTED`,
`READ_ONLY`, `XFDF_INVALID`, `TOO_LARGE`, `BUSY`, `INTERNAL`, and from the wrapper `TIMEOUT`,
`DESTROYED`, `FETCH_FAILED`. Messages are English and meant for logs, not for users.
