# Free PDF SDK

An embeddable, white-label PDF viewer and annotator for the web. It is
[pdfcraft](https://github.com/storytold/pdfcraft) (Rust) compiled to WebAssembly, behind a small
typed JavaScript API. Each viewer runs in its own iframe on your origin and talks to your page
over a versioned postMessage protocol.

```js
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer"), {
  baseUrl: "/free-pdf-sdk/0.1.0/",
  document: { url: "/files/artwork.pdf" },
  mode: "annotate",
  author: "Priya Raman",
  theme: "system",
  disabledElements: ["download", "print"],
});

await viewer.unlock(ownerPassword);            // owner-protected AES-256 files
const n = await viewer.importXfdf(storedXfdf);  // ComPDF XFDF included
const comments = await viewer.getAnnotations(); // exact count, authors, pages, replies
const xfdf = await viewer.exportXfdf();         // author in `title`
viewer.on("annotationsChanged", ({ count }) => console.log(count));
```

* **Modes**: `view` (truly read-only) and `annotate` (notes, free text, callouts, markup, shapes,
  ink, replies, Fill & Sign signatures, measurement).
* **Encrypted PDFs**: opens owner-password-protected AES-256 files, reports that commenting is
  not allowed, and allows it after `unlock(ownerPassword)`.
* **ComPDF compatible XFDF** in and out; see [docs/migrating-from-compdf.md](docs/migrating-from-compdf.md).
* **Isolated**: one iframe per viewer, no global CSS or keyboard capture, several viewers per page.
* **Self-hosted**: no licence key, no network calls, no COOP/COEP headers.

## Documentation

| | |
|---|---|
| [docs/api.md](docs/api.md) | The JavaScript API: options, methods, events, errors, `disabledElements` |
| [docs/integration.md](docs/integration.md) | Hosting (headers, caching, Firebase), bundlers, a Vue 3 component |
| [docs/migrating-from-compdf.md](docs/migrating-from-compdf.md) | ComPDF WebViewer → Free PDF SDK, XFDF compatibility and known losses |
| [docs/architecture.md](docs/architecture.md) | How it is built, the RPC protocol, the pdfcraft patches |
| [docs/updating-upstream.md](docs/updating-upstream.md) | Taking new pdfcraft releases |

## Building

Requirements: Rust (stable, with the `wasm32-unknown-unknown` target), Node 20+, and
`wasm-bindgen-cli` at the version in `Cargo.lock`:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
npm ci; npm --prefix js ci
./scripts/build.ps1            # -> dist/free-pdf-sdk-<version>/  (-Zip for a zip)
```

While the pdfcraft fork commit named in `Cargo.lock` is not yet pushed, add
`-LocalFork D:\path\to\pdfcraft` (see [docs/updating-upstream.md](docs/updating-upstream.md)).

Run the example: `node scripts/serve.mjs` and open http://127.0.0.1:8780/
(`?mode=view`, `?theme=dark`, `?doc=protected-aes256-owner.pdf`).

Tests: `cargo test -p fps-core -p fps-brand -p fps-tests` and
`npx playwright test -c e2e/playwright.config.ts` (after a build).

## Branding

The name, logos, accent colour and About text come from [brand/](brand/README.md). Replace the
placeholder SVGs and `brand/brand.toml`, then rebuild.

## Licence

Free PDF SDK is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option. It is based on pdfcraft (MIT OR Apache-2.0); see [NOTICE](NOTICE) and
[third-party/](third-party/README.md) for pdfcraft's licence and the fonts, icons and data the
viewer embeds. "PdfCraft" and "ArtCraft" are trademarks of the ArtCraft Team; this project is not
affiliated with or endorsed by them.
