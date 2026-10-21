# Writing the docs

Every page in this folder is one documentation page. `site/build.mjs` renders it to
`/docs/<slug>/` (HTML for people) and copies it to `/docs/<slug>.md` (Markdown for AI agents);
`/llms.txt` lists every page and `/llms-full.txt` concatenates them. The file name is the slug
(`react.md` → `/docs/react/`). This README is not published.

## Front matter

Every page starts with:

```
---
title: React
description: One sentence for search results and the llms.txt index.
section: Frameworks
order: 30
---
```

Sections, in sidebar order: `Get started`, `Configure`, `Frameworks`, `Guides`, `Reference`.
`order` sorts pages inside the whole sidebar (10, 20, 30...).

## Style

- Write for a developer integrating the SDK into a web app. Lead with what to do, then why.
- Short paragraphs, task-shaped headings ("Open an encrypted PDF"), working code first.
- Every code sample must be correct against the source: `js/src/free-pdf-sdk.ts` (core API),
  `js/src/element.ts`, `react.ts`, `vue.ts`, `svelte.ts`, `controller.ts` (integrations),
  `crates/core/src/features.rs` and `docs/reference/config.json` (modes, features, ui options).
  Never invent options, methods, events or error codes.
- Fenced code blocks name their language (`ts`, `tsx`, `vue`, `svelte`, `html`, `json`, `bash`).
- Link other pages as `/docs/<slug>/`. Use the package name `free-pdf-sdk`.
- The product is "Free PDF SDK". It is built on the open-source pdfcraft engine; mention that
  only where it matters (licences), never as branding.
- No marketing superlatives, no claims you can't point to in the code (sizes, speeds).

## Generated tables

Write these markers on a line of their own; the build replaces them with Markdown tables from
`docs/reference/config.json`, so they never drift from the code:

- `<!-- reference:modes -->` modes and their features
- `<!-- reference:features -->` every feature with its category and the modes that include it
- `<!-- reference:ui -->` the `ui` options
- `<!-- reference:panels -->` the panel names for `ui.panels`
- `<!-- reference:elements -->` the `disabledElements` ids
