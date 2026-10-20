# Architecture

Free PDF SDK is a white-label, embeddable PDF viewer and annotator for the web. It is
[pdfcraft](https://github.com/storytold/pdfcraft) (Rust, egui, MIT OR Apache-2.0) compiled to
WebAssembly, wrapped in a typed JavaScript API. This document describes how it is built, how the
host page and the viewer talk, and how pdfcraft updates flow in.

## Overview

```
 host page (e.g. a Vue 3 app)                             same origin
 ┌────────────────────────────────────────────┐
 │ import { FreePdf } from "free-pdf-sdk"      │
 │ viewer = await FreePdf.init(el, options)    │      postMessage (protocol v1)
 │   js/ wrapper: RPC client, typed API ───────┼──────────────┐ requests ▼  ▲ replies, events
 └────────────────────────────────────────────┘              │
 ┌────────── <iframe src="<baseUrl>/viewer/index.html#ch=…"> ┴───────────────────────┐
 │ crates/viewer (wasm32): bridge.rs (RPC server) → host.rs (Viewer: eframe::App)    │
 │   wraps pdfcraft-ui-egui::PdfCraftApp, calls fps-core operations between frames    │
 │ crates/core: protocol + validation, XFDF reading, document operations (engine)     │
 │ pdfcraft crates (git deps, one pinned commit, patched by the fork branch)          │
 └────────────────────────────────────────────────────────────────────────────────────┘
```

* **One iframe per viewer.** Each viewer is an isolated browsing context: the egui canvas
  cannot read the host's CSS or steal its keyboard focus, and several viewers on one page don't
  share state. The iframe is on the host's own origin (the host serves the SDK folder), which
  lets the protocol check origins strictly.
* **The wrapper owns the network.** `document.url` is fetched by the wrapper in the host page,
  with the host's credentials and CORS rules; the viewer only ever receives bytes.
* **No handwritten logic in the iframe page.** `viewer/index.html` only boots the wasm.

## Repository layout

| Path | What |
|---|---|
| `crates/core` (`fps-core`) | Protocol types and validation, `disabledElements`, XFDF reading and threading, document operations over `pdfcraft-engine`. No egui: tested natively and fast. |
| `crates/viewer` (`fps-viewer`) | The wasm binary: `bridge.rs` (postMessage), `host.rs` (wraps `PdfCraftApp`, dispatches requests, emits events), `branding.rs`. Its pdfcraft/egui dependencies are wasm-only, so native workspace builds stay light. |
| `crates/brand` (`fps-brand`) | Reads `brand/` (name, accent, About text, SVG logos) at compile time. |
| `brand/` | Replaceable placeholder logos and `brand.toml`. |
| `viewer/index.html` | Template of the iframe page; `build.ps1` fills in the hashed file names. |
| `js/` | The npm package `free-pdf-sdk`: TypeScript, ESM + `.d.ts`. |
| `tests/` (`fps-tests`) | Synthetic fixtures (PDFs generated in code, ComPDF-style XFDF) and integration tests. |
| `e2e/` | Playwright tests of the public API on `examples/vanilla`, headless Chromium. |
| `scripts/` | `build.ps1`, `update-upstream.ps1`, `release.ps1`, `serve.mjs`, `collect-notices.mjs`. |
| `third-party/` | Licences and notices of everything the wasm embeds (generated). |

`crates/core` exists in addition to the requested layout so that the protocol, validation and
document logic are unit-tested natively in seconds, without compiling eframe and wgpu.

## pdfcraft as a dependency

* pdfcraft is never copied. `Cargo.toml` depends on `pdfcraft-ui-egui`, `pdfcraft-engine`,
  `pdfcraft-render`, `pdfcraft-xfdf`, `pdfcraft-cos` (and others) as git dependencies on
  `https://github.com/storytold/pdfcraft`, all at one full commit (`rev = "<40 hex>"`).
* pdfcraft's own root `[patch.crates-io]` (its vendored hayro, hayro-interpret, hayro-syntax,
  hayro-jbig2, lopdf, winit, egui-wgpu, vello_common) is not inherited by git dependents, so our
  workspace repeats it, taking those crates from the same pdfcraft commit.
* Hooks pdfcraft lacks live as small commits on the fork branch `free-pdf-sdk/patches` of
  `https://github.com/GeorgeMilroy/pdfcraft`, created from `upstream/main`. A
  `[patch."https://github.com/storytold/pdfcraft"]` section points every pdfcraft crate at that
  branch's commit. `Cargo.lock` is committed.
* Before the fork commit is pushed, `scripts/dev-env.ps1 -LocalFork <clone>` makes Cargo fetch
  it from a local clone (process-scoped git `insteadOf`); the manifests always name the real URL.

### Patches (fork branch `free-pdf-sdk/patches`)

Each patch is one concern, an opt-in hook whose default leaves pdfcraft unchanged, and written
so it can go upstream as a PR. We first checked what `PdfCraftApp`'s public API allows from
outside; these two could not be done from outside:

| Patch | Why it must be inside pdfcraft |
|---|---|
| `free-pdf-sdk: add a Brand hook …` | Logos are `include_bytes!` in widgets.rs; "PdfCraft"/"ArtCraft" are literals in the window title, About, Home, the tab strip's Discord button, Help links and the translated catalogs. `brand::set(Brand)` replaces them; `Brand::default()` is PdfCraft. Translations are renamed at lookup time, so no catalog changes. |
| `free-pdf-sdk: add a UI profile …` | There is no locked read-only mode (`Mode::Read` can be left) and no way to hide commands, tools or tool groups. `PdfCraftApp::profile` hides ids and locks edits; the default profile changes nothing. |

Everything else the API needs is reachable from outside (`fps-viewer` wraps `PdfCraftApp` in its
own `eframe::App` and runs requests between frames):

| Need | How |
|---|---|
| open bytes, user password | `open_bytes`, `password_prompt` + `submit_password` |
| owner unlock, permissions | `session.unlock`, `Document::allows_*`, `can_unlock` |
| XFDF import | `apply_edit(Edit::ImportData)` (one undo step with `replace`) |
| XFDF export, listing | `session.export_data(DataFormat::Xfdf)`, parsed by `fps-core::xfdf` |
| delete all | `apply_edit(Edit::Batch[DeleteAnnotation…])`, highest index first |
| author override | `comment_prefs.author`, set after settings are restored and after each load |
| theme, locale, page, zoom | `set_theme_preference`, `set_option("language"|"page"|"zoom")`, `DocView::set_fit` |
| dirty, change events | `first_dirty()`, `Document::edit_generation()` + a comments fingerprint |
| UI Save button | `host_save` → event `saveRequested` (never a download) |
| document loaded / error | the result of `open_bytes`, polled until the document is active |

## The RPC protocol (version 1)

All messages are plain objects sent with `postMessage`:

```ts
{ fps: 1, ch: "<channel id>", kind: "hello" | "req" | "res" | "evt", ... }
```

| kind | direction | fields |
|---|---|---|
| `hello` | viewer → host | `info: { sdk, pdfcraftRev, protocol }`, once the viewer listens |
| `req` | host → viewer | `id: uint32`, `method: string`, `params: object` (binary in `params.data`, transferred) |
| `res` | viewer → host | `id`, `ok: true, result` or `ok: false, error: { code, message }` (binary results are a transferred `Uint8Array`) |
| `evt` | viewer → host | `name`, `data` |

**Channel.** The wrapper generates a 128-bit random channel id and opens the iframe at
`viewer/index.html#ch=<id>`. Every message carries it; each side drops messages for other
channels, so several viewers on one page never cross.

**Origin and source checks.**

* Viewer: accepts a message only if `event.origin === location.origin`, `event.source ===
  window.parent`, `ch` matches and `kind === "req"`. It posts only to `window.parent` with its own
  origin as the target origin.
* Wrapper: accepts a message only if `event.origin` is the SDK's origin, `event.source ===
  iframe.contentWindow` and `ch` matches. It refuses a `baseUrl` on another origin.

**Versioning.** `fps` is the protocol version. A request with another version gets
`UNSUPPORTED_PROTOCOL`; the wrapper warns when the viewer's SDK version differs from its own.
Additive changes (new methods, new optional params, new result fields) keep version 1.

**Validation.** Every request is untrusted. `fps-core::protocol::parse_request` checks each
parameter's type, size and range (names ≤ 512 chars, author ≤ 200, password ≤ 1024, XFDF ≤ 32 MiB,
PDF ≤ 512 MiB, `page` ≥ 1, zoom 10–6400 %, element ids from the documented list) and never panics.
XFDF is parsed without DTDs and with a nesting limit.

**Methods.** `configure`, `load`, `unlock`, `getPermissions`, `importXfdf`, `exportXfdf`,
`getAnnotations`, `deleteAllAnnotations`, `setAuthor`, `setTheme`, `setMode`,
`setDisabledElements`, `setLocale`, `getPageCount`, `goToPage`, `setZoom`, `isDirty`, `save`,
`version`. Requests run in order, between frames; `load` replies once the document is open.

**Events.** `documentLoaded {pageCount, permissions}`, `documentError {code, message}`,
`annotationsChanged {count}`, `dirtyChanged {dirty}`, `error {code, message}`,
`saveRequested {name, saveAs, data}`. The wrapper adds `ready` after `configure`.

**Errors.** `BAD_REQUEST`, `UNKNOWN_METHOD`, `UNSUPPORTED_PROTOCOL`, `NOT_READY`, `NO_DOCUMENT`,
`LOAD_FAILED`, `NEEDS_PASSWORD`, `WRONG_PASSWORD`, `NOT_PERMITTED`, `READ_ONLY`, `XFDF_INVALID`,
`TOO_LARGE`, `BUSY`, `INTERNAL`; the wrapper adds `TIMEOUT`, `DESTROYED`, `FETCH_FAILED`. The
wrapper rejects with `FreePdfError { code, message }`.

## Modes

* **view**: read-only. The API refuses `importXfdf` and `deleteAllAnnotations` with `READ_ONLY`;
  the UI profile hides every editing command and tool and refuses every edit, undo, redo and save
  in pdfcraft itself.
* **annotate**: comments (notes, free text, callouts, markup), shapes, ink, replies, Fill & Sign
  (drawn and typed signatures) and measurement. Page, content, form and security changes are
  hidden and refused.

## XFDF and ComPDF compatibility

* Import goes through pdfcraft's XFDF reader (`pdfcraft-xfdf`), which maps `title` to the author
  and reads 0-based `page`, `name`, `inreplyto`, colours, ink, vertices, callouts and pop-ups.
* Export is pdfcraft's XFDF writer: author in `title`. The SDK's annotation list is read back
  from that export, so `getAnnotations()` counts exactly what `exportXfdf()` stores: top-level
  comments, with replies nested. `xfdf.includes("<annots")` is not a test for comments (pdfcraft
  always writes `<annots>`).
* Known losses are listed in docs/migrating-from-compdf.md.

## Security notes

* The viewer never downloads, prints or opens files on its own when those elements are disabled;
  the UI Save hands bytes to the host.
* Passwords travel only over the same-origin channel and are not stored.
* No `eval`, no remote code: the iframe loads only the SDK's own hashed files.

## Build

`scripts/build.ps1`: `cargo build -p fps-viewer --target wasm32-unknown-unknown --release` →
`wasm-bindgen --target web` (the CLI version must match `Cargo.lock`) → `wasm-opt -Oz` →
content-hashed names → `viewer/index.html` → `tsc` → licences, notices, `version.json` →
`dist/free-pdf-sdk-<version>/`. See docs/integration.md for hosting.
