---
title: Docs for AI agents
description: Machine-readable documentation for coding assistants and LLM agents that integrate Free PDF SDK.
section: Reference
order: 75
---

# Docs for AI agents

Free PDF SDK publishes its documentation in forms that coding assistants and LLM agents can
read directly. Point your assistant at one of these URLs (relative to this site):

| URL | What it is |
|---|---|
| `/llms.txt` | An index of every documentation page with a one-line summary, in the [llms.txt](https://llmstxt.org) format. Start here. |
| `/llms-full.txt` | Every page in one Markdown file, for agents that prefer a single document. |
| `/docs/<page>.md` | Any page as Markdown, for example `/docs/configuration.md` or `/docs/react.md`. |
| `/schema/viewer-options.schema.json` | A JSON Schema of the options object (`mode`, `features`, `ui`, `disabledElements`...). Validate generated configurations against it. |
| `/reference/config.json` | Modes, features, interface options and element ids, as data. |

## Rules for agents integrating the SDK

Give these to your assistant along with the docs; they prevent the common mistakes.

1. **Serve the viewer from the page's own origin.** Run
   `npx @mecorelabs/free-pdf-sdk copy-viewer <public folder>/free-pdf-sdk` after installing, and pass that
   folder's URL as `baseUrl`. A `baseUrl` on another origin is rejected (`BAD_REQUEST`).
2. **Give the container a height.** The viewer fills its element; an element with no height
   shows nothing.
3. **Pick a `mode`, then adjust with `features`.** Don't hide things with CSS or
   `disabledElements` when a feature switch exists: a feature that is off disappears from every
   menu, toolbar, panel and shortcut, and its actions are refused.
4. **Use only documented names.** Unknown feature keys, `ui` options, panel names or element ids
   are rejected with `BAD_REQUEST`. The lists are in `/reference/config.json`.
5. **Render on the client only.** In Next.js, Nuxt or SvelteKit, mount the viewer in a
   client-only component; it needs `window`, WebGL and an iframe.
6. **Every method is async and rejects with `FreePdfError`** (`error.code` is one of the
   documented codes). Await calls and handle `READ_ONLY`, `NOT_PERMITTED` and `NO_DOCUMENT`.
7. **Store comments as XFDF.** `exportXfdf()` returns a string to save; `importXfdf(xfdf,
   { replace: true })` restores it. Authors are in the `title` attribute.

## A minimal, correct integration

```ts
import { FreePdf, FreePdfError } from "@mecorelabs/free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",          // same origin as this page
  mode: "annotate",                   // annotations only, no other edits
  features: { measurement: false },   // fine-tune the mode
  ui: { panels: { layers: false } },  // show or hide parts of the interface
  author: "Ada Lovelace",
  document: { url: "/files/proof.pdf" },
});

viewer.on("annotationsChanged", async () => {
  const xfdf = await viewer.exportXfdf();
  await fetch("/api/comments", { method: "PUT", body: xfdf });
});

try {
  await viewer.importXfdf(savedXfdf, { replace: true });
} catch (e) {
  if (e instanceof FreePdfError && e.code === "NOT_PERMITTED") {
    // The document's security forbids comments: ask for the owner password, then viewer.unlock(password).
  }
}
```
