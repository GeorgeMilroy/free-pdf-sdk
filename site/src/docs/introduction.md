---
title: Introduction
description: What Free PDF SDK is, what it can do, and how the in-browser viewer fits into your web app.
section: Get started
order: 10
---

# Introduction

Free PDF SDK is an embeddable PDF viewer, annotator and editor for web apps. You add it to a
page with one call (or one component), choose what your users are allowed to do, and talk to it
through a typed, promise-based JavaScript API. Documents are processed in the browser with
WebAssembly; the SDK does not upload them anywhere.

```ts
import { FreePdf } from "@mecorelabs/free-pdf-sdk";

const viewer = await FreePdf.init(document.getElementById("viewer")!, {
  baseUrl: "/free-pdf-sdk/",            // the viewer files, served by your site
  mode: "annotate",                     // what users may do
  document: { url: "/files/contract.pdf" },
});

viewer.on("annotationsChanged", ({ count }) => console.log(`${count} comments`));
```

The package is `@mecorelabs/free-pdf-sdk`, licensed MIT OR Apache-2.0.

## Key capabilities

- **Viewing**: navigation, zoom, search, copy, print, bookmarks, page thumbnails, document
  properties, light and dark themes, and a translated interface.
- **Annotation**: sticky notes, text boxes, callouts, highlight, underline, strikethrough and
  squiggly markup, shapes, freehand ink, stamps, measurement (distance, perimeter, area) and
  replies with review status.
- **Fill & Sign**: typed text, checkmarks, dates, and drawn or typed signatures and initials
  placed on the page; filling in PDF forms.
- **Authoring**, when you turn it on: page organization, bookmark editing, content editing,
  form design, flattening, redaction, accessibility tagging, PDF/A conversion, digital
  signatures and document security.
- **Modes and feature switches**: start from one of eight presets and turn individual features
  on or off. A feature that is off is hidden from every menu, toolbar, panel and shortcut, and
  refused if it is reached anyway. Any configuration without an editing feature is read-only.
- **PDF layers**: the Layers panel lists the document's layers (optional content) and shows or hides each
  one with a click; printing follows what is shown.
- **Comments as data**: export and import comments as standard XFDF (author in `title`), list them with
  their replies, and react to changes through events.
- **Encrypted PDFs**: open password-protected files, including AES-256, and lift owner
  restrictions with the owner password.
- **Framework integrations**: React, Vue, Svelte, a Web Component for Angular and everything
  else, and plain JavaScript.

## Modes at a glance

Every viewer starts in a mode. The default is `view`, which is read-only.

<!-- reference:modes -->

Fine-tune any mode with `features`, and show or hide parts of the interface with `ui`. See
[Configuration](/docs/configuration/).

## How it works

- **Your site serves the viewer.** `npx @mecorelabs/free-pdf-sdk copy-viewer` copies the viewer files (an
  HTML page, a JavaScript file and a WebAssembly module) into a folder of your site. There is no
  CDN and no server component: they are static files.
- **Each viewer is an iframe on your own origin.** `FreePdf.init` creates an `<iframe>` that
  loads `<baseUrl>/viewer/index.html` inside the element you give it. The folder must be on the
  same origin as your page.
- **The page and the viewer talk over `postMessage`.** The wrapper you import sends requests to
  the iframe over a versioned protocol with strict origin and channel checks, and turns the
  replies into promises and the viewer's notifications into events.
- **Your page fetches the PDF, the viewer gets bytes.** A `document.url` is fetched in your
  page, with your page's cookies, headers and CORS rules; the wrapper hands the file's bytes to
  the viewer.
- **Everything runs in the browser.** The viewer is a WebAssembly build of an open-source PDF
  engine and draws its interface with WebGL. Saving returns the PDF to your code as a
  `Uint8Array`; storing it is up to you.

## Where to go next

- [Getting started](/docs/getting-started/): install the package, serve the viewer and open
  your first PDF.
- [Configuration](/docs/configuration/): modes, features, interface options and
  `disabledElements`.
- Frameworks: [plain JavaScript](/docs/vanilla-js/), [React](/docs/react/), [Vue](/docs/vue/),
  [Svelte](/docs/svelte/), [Angular](/docs/angular/) and the [Web Component](/docs/web-component/).
- [Annotations and XFDF](/docs/annotations-and-xfdf/): store, restore and list comments.
- [API reference](/docs/api/): every option, method, event and error code.
- [Hosting and security](/docs/hosting-and-security/): headers, caching, Content-Security-Policy
  and examples for common hosts.
- [Encrypted PDFs](/docs/encrypted-pdfs/), [Theming and localization](/docs/theming-and-localization/).
- [FAQ and troubleshooting](/docs/faq/) when something doesn't show up.
- [Docs for AI agents](/docs/ai-agents/) if a coding assistant is doing the integration.
