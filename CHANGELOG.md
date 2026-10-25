# Changelog

All notable changes to Free PDF SDK are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html): the JavaScript API and the
postMessage protocol are the public surface.

## [Unreleased]

pdfcraft updated to 0.14.0 and later upstream changes. No change to the JavaScript API or the
postMessage protocol.

### Fixed

- Taps and clicks on the page are read once per frame, so a tap on a form field, check box or
  open editor works, and nothing drawn over a page (a dialog, a panel being dragged) lets a
  click through to it.
- Rendering follows the PDF specification more closely: luminosity soft masks in CMYK groups,
  Separation `/All` colours, twelve-ink DeviceN colours, Type 3 invisible text, tiling patterns
  at every zoom, symbolic TrueType fonts and stand-ins for fonts that aren't embedded.
- Comments without their own appearance are drawn: sticky-note icons (upright on rotated pages),
  named stamps, border dashes, and line comments' leader lines and captions.
- Check boxes and radio buttons without an appearance are drawn, with their caption and in the
  field's text colour; comb fields show their cells, and fields that don't scroll stop taking
  text when full.
- The viewer's Comments panel threads replies by the comment they answer (`/IRT`) and counts
  comments as Acrobat counts them. (`getAnnotations` and `annotationsChanged` still group replies
  by `/NM` as before.)
- A document opens at its own opening destination (page, position or named destination).
- Redaction: clearing marks leaves the page as it was, form fields are removed even from an
  empty form, and images drawn through a form without resources are kept.
- Typed Password-field values are never written to a saved, extracted or signed file.
- A static XFA form whose data fills thousands of fields opens in seconds.
- Many robustness fixes from pdfcraft's code audits: bounded xref streams and JBIG2 data, a CMap
  can no longer overflow the stack, and stricter XFDF, FDF and FormCalc parsing.
- Security advisories in dependencies: crypto-bigint 0.7.6 (RUSTSEC-2026-0387) and ctutils 0.4.5
  (RUSTSEC-2026-0385).

### Added

- Document Properties: a **Custom** tab for custom properties (editable with the
  `editContent` feature) and a **Space Audit** tab.
- Bookmarks show their title colour and style, and can set them (`bookmarks` feature).
- The Pages panel has a **Page tools** menu (insert, rotate, extract, delete), and page
  thumbnails a **Split document…** item; each shows only where its command is allowed.
- The Attachments panel: double-click an attachment (other than a PDF) to save it, or
  right-click it for Save and Delete (`download` feature to save, `editContent` to delete one of
  the document's own attachments).
- **Flatten comments** in the page's right-click menu and the Comments panel's menu
  (`flatten` feature).
- Measurement: type or pick an architect's 1:N drawing scale, applied at once.
- The arrow keys move the selected comment (where its feature is on); added text boxes and images
  can be pasted.
- Forms: an editable combo box takes a typed value, and the date picker has a **Today** button.
- Extract pages takes typed page ranges.
- PDF Optimizer can target a PDF version and choose whether to use object streams (`download`
  feature).
- All tools expands to a full, searchable view listing the tools the configuration allows.
- Persian interface translation.

### Changed

- The secured-document notice has a "Don't show again" button.
- pdfcraft's new Edit ▸ **Attach a file** is not offered: it needs a native file picker. The
  Attach file comment tool is unchanged.

### Based on

- pdfcraft upstream `71b47b519d2c295c2068dc89a8d75690fc7dd7e5` (0.14.0 and later) with the
  `free-pdf-sdk/patches` commits rebased onto it (exact commit in `version.json`).

## [0.2.2] - 2026-10-23

pdfcraft updated to 0.13.0 (through 0.12.0). No change to the JavaScript API or the postMessage
protocol.

### Fixed

- Applying redactions and flattening work on a copy: a page refused part way leaves the
  document as it was.
- Many robustness fixes from pdfcraft's code audits: decoding is bounded across all filter
  stages, image export refuses a bad raster instead of failing, and form, XFA date and signature
  parsing are stricter.
- Form fields show their tooltip on hover.

### Added

- View ▸ Page display ▸ **Show gaps between pages** (on by default): turn it off for a gapless
  continuous view. Always available; hide it with the `view.page_gaps` id.
- Lines, connected lines, polygons and clouds can be reshaped by dragging their ends and corners
  (`shapes` feature).
- Remove a document's PDF/A identification from the PDF/A dialog (`standards` feature).
- Header and footer can shrink the page content out of their way, and use Times or Courier.
- A new bookmark goes where the reader is; Pages panel thumbnails can be resized.
- French and Brazilian Portuguese interface translations completed; Polish and others extended.

### Changed

- The "Save changes?" prompt can be answered from the keyboard: the arrow keys and Tab move
  between its buttons and Enter or Space press the chosen one. When the configuration doesn't
  allow saving, the choice starts on Cancel and never lands on Save.

### Based on

- pdfcraft upstream `0dc77fe4ed8810b9e9f25b9dd050afae662c062a` (0.13.0) with the
  `free-pdf-sdk/patches` commits rebased onto it (exact commit in `version.json`).

## [0.2.1] - 2026-10-22

pdfcraft updated to 0.11.0 and later upstream changes. No change to the JavaScript API or the
postMessage protocol.

### Fixed

- Redaction: "Remove hidden information" and "Sanitize document" now evaluate
  optional-content membership dictionaries (OCMDs) when they remove hidden layers, so content a
  document actually shows is kept and content it hides is removed; malformed visibility data is
  treated conservatively.
- Rendering reads each object stream's index once instead of again for every object in it.
- PNG export is 3-5x faster for opaque pages, at the same file size.

### Added

- Prepare a form ▸ **Calculation order** (`formDesign` feature): a page to read, check, fix and
  set the order calculated fields run in. Renaming a field renames it in the calculations that
  read it.
- Prepare a form ▸ **Text field table** (`formDesign` feature): draw a grid of text fields (rows
  and columns) as one edit.
- Norwegian Bokmål interface and date language.

### Changed

- The mode bar has no Read tab, as in Acrobat; Read mode stays a View command, and a read-only
  configuration still pins the viewer to it.
- The Edit panel and Format text are laid out as Acrobat's; the Tools panel has no tab strip
  while docked on its own; Organize pages remembers its thumbnail size.

### Based on

- pdfcraft upstream `88dad98dd478d255399b1405b33765f4da542704` (0.11.0 and later main) with the
  `free-pdf-sdk/patches` commits rebased onto it (exact commit in `version.json`). The fork's
  profile gates also cover the new Edit panel.

## [0.2.0] - 2026-10-21

### Changed

- The npm package is published under the mecorelabs organisation as `@mecorelabs/free-pdf-sdk`
  (imports become `@mecorelabs/free-pdf-sdk`, `@mecorelabs/free-pdf-sdk/react` and so on). The
  CLI keeps its name: `free-pdf-sdk copy-viewer`.

### Added

- The viewer's About dialog no longer has the Contributors and Models tabs: they listed
  pdfcraft's project contributors and the AI models used to build it. pdfcraft's attribution
  stays in `NOTICE` and `third-party/` (pdfcraft fork: `Brand::credits`).
- `loader` option: your own SVG, GIF or other image as the loading animation while the viewer
  starts and while a document opens (`loader` attribute on `<free-pdf-viewer>`). By default a
  built-in spinner now covers the viewer until it is ready; `loader: false` turns it off.
- No `ocr` feature: the browser build has no OCR models, so text recognition is not offered and the viewer's OCR tools are hidden.
- The sample document's floor plan has four layers (Structure, Furniture, Electrical, Dimensions) for the Layers panel.
- **Configuration model.** Eight modes (`view`, `review`, `annotate`, `fill`, `sign`, `organize`,
  `edit`, `full`), each a preset of 31 features, plus per-feature overrides (`features`) and
  interface options (`ui`: Tools panel, tools rail, quick toolbar, right rail, zoom and page
  navigation, menu, search box, individual panels, layout lock, planned tools, external
  links). A feature that is off is hidden from every menu, submenu, context menu, toolbar,
  panel, dialog and keyboard shortcut, and its actions are refused if reached (pdfcraft fork:
  every command, tool group, document edit and annotation kind belongs to one feature).
- API: `setFeatures`, `setUi`, `getConfig`; `setMode` accepts every mode. Unknown feature,
  option and panel names are rejected with `BAD_REQUEST`.
- Framework integrations: `@mecorelabs/free-pdf-sdk/react` (component and ref handle),
  `@mecorelabs/free-pdf-sdk/vue` (component), `@mecorelabs/free-pdf-sdk/svelte` (action) and `@mecorelabs/free-pdf-sdk/element`
  (`<free-pdf-viewer>` Web Component, for Angular and any other framework), all built on a shared
  controller that turns option changes into API calls.
- npm package with the viewer files and a CLI: `npx @mecorelabs/free-pdf-sdk copy-viewer <folder>`
  (`scripts/build.ps1` stages it in `dist/npm`).
- The landing site (`site/`, Firebase Hosting): home page, live demo running the real SDK, and
  documentation for people and for AI agents (`/llms.txt`, `/llms-full.txt`, Markdown pages, a
  JSON Schema of the options). Tables come from `docs/reference/config.json`, which a test keeps
  in step with the code.
- Brand: the Free PDF SDK logo (vector, traced from the artwork) and brand colours in the viewer.
- A synthetic demo document (cover with a floor plan, an approval form, a dimensioned
  specification) in `tests/fixtures/demo.pdf`.

### Changed

- `importXfdf` and `deleteAllAnnotations` reject with `READ_ONLY` whenever the configuration
  has no annotation feature (not only in `view` mode).
- Any configuration without a document-changing feature is read-only (the viewer stays in Read
  mode), whatever its mode.

### Added (earlier in 0.2.0)

- Annotate mode: a closed Tools drawer (or one Read mode hides) leaves a slim rail on its side,
  the left by default, to pull it back out: the chevron reopens it as it was, and each tool
  group's coloured icon opens that group (pdfcraft fork patch).

### Fixed

- Annotate mode: the Tools drawer lists all four tool groups (Fill & Sign, Add comments, Add a
  stamp, Measure objects). Measure objects was hidden behind "View more" (pdfcraft fork patch).
  The drawer now opens on that list instead of inside Add comments.

### Based on

- pdfcraft upstream `6fb6b7cf6fd8dc105fdd56a764d207cff6140694` (0.10.0) with the
  `free-pdf-sdk/patches` commits (exact commit in `version.json`).

## [0.1.0] - 2026-10-20

### Added

- `FreePdf.init(element, options)`: an iframe-isolated viewer on the host's origin, with
  `baseUrl`, `document` (`url` or `data`), `mode` (`view` | `annotate`), `author`, `theme`,
  `password`, `disabledElements`, `locale`, `timeoutMs` and early `listeners`.
- Methods: `load`, `unlock`, `getPermissions`, `importXfdf`, `exportXfdf`, `getAnnotations`,
  `deleteAllAnnotations`, `setAuthor`, `setTheme`, `setMode`, `setDisabledElements`,
  `setLocale`, `getPageCount`, `goToPage`, `setZoom`, `isDirty`, `save`, `version`, `destroy`,
  `on`/`off`.
- Events: `ready`, `documentLoaded`, `documentError`, `annotationsChanged`, `dirtyChanged`,
  `error`, `saveRequested`.
- postMessage RPC protocol version 1: request ids, typed `{ code, message }` errors, channel ids,
  origin and source checks on both sides, validated inputs.
- Standard XFDF import and export (author in `title`), exact comment counts, replies.
- Owner-password-protected AES-256 PDFs: permissions reported, `unlock(ownerPassword)`.
- Locked view mode and comments-only annotate mode (pdfcraft UI profile patch).
- White-label branding from `brand/` (pdfcraft Brand patch); no ArtCraft or PdfCraft marks.
- Build (`scripts/build.ps1`), upstream update (`scripts/update-upstream.ps1`) and release
  (`scripts/release.ps1`) scripts; CI; Playwright e2e tests; docs.

### Based on

- pdfcraft upstream `6fb6b7cf6fd8dc105fdd56a764d207cff6140694` (0.10.0) with the
  `free-pdf-sdk/patches` commits (exact commit in `version.json`).

[Unreleased]: https://github.com/GeorgeMilroy/free-pdf-sdk/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/GeorgeMilroy/free-pdf-sdk/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/GeorgeMilroy/free-pdf-sdk/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/GeorgeMilroy/free-pdf-sdk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/GeorgeMilroy/free-pdf-sdk/releases/tag/v0.1.0
