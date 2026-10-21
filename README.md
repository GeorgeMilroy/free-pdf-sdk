# Free PDF SDK

An embeddable, white-label PDF viewer, annotator and editor for the web. It is
[pdfcraft](https://github.com/storytold/pdfcraft) (Rust) compiled to WebAssembly, behind a small
typed JavaScript API with React, Vue, Svelte and Web Component integrations. Each viewer runs in
its own iframe on your origin and talks to your page over a versioned postMessage protocol;
documents never leave the browser.

```js
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer"), {
  baseUrl: "/free-pdf-sdk/",
  document: { url: "/files/artwork.pdf" },
  mode: "annotate",                              // annotations only, no other edits
  features: { stamps: false, print: false },     // fine-tune the mode
  ui: { panels: { layers: false } },             // show or hide parts of the interface
  author: "Priya Raman",
});

await viewer.unlock(ownerPassword);            // owner-protected AES-256 files
const n = await viewer.importXfdf(storedXfdf);  // standard XFDF from any PDF tool
const comments = await viewer.getAnnotations(); // exact count, authors, pages, replies
const xfdf = await viewer.exportXfdf();         // author in `title`
viewer.on("annotationsChanged", ({ count }) => console.log(count));
```

* **Modes and features**: eight modes (`view`, `review`, `annotate`, `fill`, `sign`,
  `organize`, `edit`, `full`) and 31 feature switches. A feature that is off is hidden from
  every menu, toolbar, panel, context menu and shortcut, and refused if reached.
* **Frameworks**: `free-pdf-sdk/react`, `free-pdf-sdk/vue`, `free-pdf-sdk/svelte`, and
  `free-pdf-sdk/element` (`<free-pdf-viewer>`) for Angular and everything else.
* **Encrypted PDFs**: opens owner-password-protected AES-256 files, reports what is allowed, and
  lifts restrictions with `unlock(ownerPassword)`.
* **Standard XFDF** in and out, author in `title`.
* **Isolated and self-hosted**: one iframe per viewer, no licence key, no network calls.

## Documentation

The full documentation is the website in [site/](site/README.md) (docs for people, plus
`/llms.txt` for AI agents). Its pages are in [site/src/docs/](site/src/docs/). In this repository:

| | |
|---|---|
| [docs/reference/config.json](docs/reference/config.json) | Modes, features, interface options, element ids (generated, tested) |
| [docs/architecture.md](docs/architecture.md) | How it is built, the RPC protocol, the pdfcraft patches |
| [docs/updating-upstream.md](docs/updating-upstream.md) | Taking new pdfcraft releases |
| [docs/integration.md](docs/integration.md), [docs/api.md](docs/api.md) | The 0.1.0 docs; superseded by the site's pages |

## Building

Requirements: Rust (stable, with the `wasm32-unknown-unknown` target), Node 20+, and
`wasm-bindgen-cli` at the version in `Cargo.lock`:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
npm ci; npm --prefix js ci
./scripts/build.ps1            # -> dist/free-pdf-sdk-<version>/ and dist/npm/  (-Zip for a zip)
node site/build.mjs            # -> site/public/ (the website, with the SDK and the live demo)
```

While the pdfcraft fork commit named in `Cargo.lock` is not yet pushed, add
`-LocalFork D:\path\to\pdfcraft` (see [docs/updating-upstream.md](docs/updating-upstream.md)).

Run the example: `npm run dev:example` and open http://127.0.0.1:8780/
(`?mode=view`, `?mode=fill`, `?theme=dark`, `?doc=protected-aes256-owner.pdf`). Run the site:
`npm run build:site`, then `npm run preview:site:dev`. Deploy it: `npm run deploy:hosting:prod`.

Tests: `cargo test -p fps-core -p fps-brand -p fps-tests` and `npm run test:e2e` (after a build).

## Branding

The name, logos, accent colours and About text come from [brand/](brand/README.md). Replace the
SVGs and `brand/brand.toml`, then rebuild.

## Licence

Free PDF SDK is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option. It is based on pdfcraft (MIT OR Apache-2.0); see [NOTICE](NOTICE) and
[third-party/](third-party/README.md) for pdfcraft's licence and the fonts, icons and data the
viewer embeds. "PdfCraft" and "ArtCraft" are trademarks of the ArtCraft Team; this project is not
affiliated with or endorsed by them.
