# Free PDF SDK

**Show your users exactly what they need. Hide the rest.**

A free, open-source, white-label PDF viewer, annotator and editor for the web. Your users switch
PDF layers on and off with a click. You switch panels, tools and whole features on or off, and
every menu, toolbar and shortcut follows. It runs in the browser (WebAssembly), so documents never
leave your page.

[Website](https://freepdfsdk.mecorelabs.com/) · [Live demo](https://freepdfsdk.mecorelabs.com/demo/) ·
[Get started](https://freepdfsdk.mecorelabs.com/docs/getting-started/) ·
[Docs](https://freepdfsdk.mecorelabs.com/docs/) · [GitHub](https://github.com/GeorgeMilroy/free-pdf-sdk)

![The viewer in annotate mode on a floor plan, with the Layers panel open and the Furniture layer hidden](https://freepdfsdk.mecorelabs.com/assets/img/viewer-layers.png)

- **8 modes**, from read-only to full authoring, and **31 feature switches**. What's off is gone
  from every menu, toolbar, right-click menu and shortcut, not greyed out.
- **PDF layers**: show or hide structure, furniture, electrical, dimensions... in one click.
  Printing follows what is shown; the file is untouched.
- **Comments & review**: notes, callouts, highlights, shapes, ink and stamps, with replies and
  review status. Standard XFDF in and out.
- **Measurement**, **Forms, Fill & Sign**, **page organization**, **redaction**, **signature
  validation**, export to Word, images and HTML, PDF/A, accessibility checker, search and print.
- **Private by design**: an iframe on your own origin, no server, no uploads, no tracking.
  AES-256 encrypted files open in place; an owner password lifts their restrictions.
- **23 languages**, light, dark and system themes.
- **React, Vue, Svelte, Angular** (Web Component) and plain JavaScript. Fully typed.

## Install

```bash
npm install @mecorelabs/free-pdf-sdk
npx @mecorelabs/free-pdf-sdk copy-viewer public/free-pdf-sdk
```

The second command copies the viewer files into a folder your site serves. The viewer must be
served from the same origin as your page. Run it again after upgrading, or add it as a
`postinstall` script: `"postinstall": "free-pdf-sdk copy-viewer public/free-pdf-sdk"`.

## Use

```ts
import { FreePdf } from "@mecorelabs/free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",
  mode: "annotate",
  features: { stamps: false, measurement: false },
  author: "Ada Lovelace",
  document: { url: "/files/plan.pdf" },
});

viewer.on("annotationsChanged", ({ count }) => console.log(count, "comments"));
const xfdf = await viewer.exportXfdf();      // store it anywhere; importXfdf() brings it back
```

Give the element a height. Modes: `view`, `review`, `annotate`, `fill`, `sign`, `organize`,
`edit`, `full`.

### Frameworks

```tsx
// React
import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/react";
<FreePdfViewer baseUrl="/free-pdf-sdk/" mode="annotate" document={{ url: "/files/plan.pdf" }} style={{ height: "80vh" }} />
```

```vue
<!-- Vue -->
<script setup lang="ts">
import { FreePdfViewer } from "@mecorelabs/free-pdf-sdk/vue";
</script>
<template>
  <FreePdfViewer base-url="/free-pdf-sdk/" mode="annotate" :document="{ url: '/files/plan.pdf' }" style="height: 80vh" />
</template>
```

```html
<!-- Web Component: Angular and everything else -->
<script type="module">
  import { defineFreePdfViewer } from "@mecorelabs/free-pdf-sdk/element";
  defineFreePdfViewer();
</script>
<free-pdf-viewer base-url="/free-pdf-sdk/" src="/files/plan.pdf" mode="view" style="height: 80vh"></free-pdf-viewer>
```

Svelte: `import { freePdf } from "@mecorelabs/free-pdf-sdk/svelte"` (an action).

## Documentation

Guides for every framework, the full API reference and recipes:
**[freepdfsdk.mecorelabs.com/docs](https://freepdfsdk.mecorelabs.com/docs/)**. For AI coding assistants,
every page is also Markdown, indexed in [/llms.txt](https://freepdfsdk.mecorelabs.com/llms.txt), with a
[JSON Schema](https://freepdfsdk.mecorelabs.com/schema/viewer-options.schema.json) of the options.

## Support the project

Free PDF SDK is open source and free. Donations pay for bringing in each new engine release,
testing new browsers, fixing bugs and keeping the docs current:
[Donate on Ko-fi](https://ko-fi.com/K4V228W28L).

## Licence

MIT OR Apache-2.0. Built on [pdfcraft](https://github.com/storytold/pdfcraft) by the ArtCraft
Team; see `NOTICE` and `third-party/` in the package.
