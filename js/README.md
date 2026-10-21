# Free PDF SDK

An embeddable, white-label PDF viewer, annotator and editor for the web. It runs entirely in
the browser (WebAssembly): documents never leave your page.

- Modes and per-feature switches: show only what your users need (view only, annotations only,
  forms, page organization, full editing) and everything else is hidden from every menu.
- Comments, markup, shapes, ink, stamps, measurement, Fill & Sign, forms, page organization.
- Standard XFDF import and export (author in `title`); AES-256 encrypted PDFs with owner-password unlock.
- React, Vue, Svelte, Angular (Web Component) and plain JavaScript.

## Install

```bash
npm install free-pdf-sdk
npx free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies the viewer files into a folder your site serves (run it again after
upgrading). The viewer must be served from the same origin as your page.

## Use

```ts
import { FreePdf } from "free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  features: { stamps: false },
  document: { url: "/files/contract.pdf" },
});
viewer.on("annotationsChanged", ({ count }) => console.log(count));
const xfdf = await viewer.exportXfdf();
```

React: `import { FreePdfViewer } from "free-pdf-sdk/react"`. Vue: `free-pdf-sdk/vue`.
Svelte: `free-pdf-sdk/svelte`. Web Component (Angular and others): `free-pdf-sdk/element`.

Documentation: see the Free PDF SDK website (docs for people, and `llms.txt` for AI agents).

## Licence

MIT OR Apache-2.0. Built on open-source software; see NOTICE and third-party/.
