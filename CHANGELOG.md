# Changelog

All notable changes to Free PDF SDK are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html): the JavaScript API and the
postMessage protocol are the public surface.

## [Unreleased]

### Fixed

- Annotate mode: the Tools drawer lists all four tool groups (Fill & Sign, Add comments, Add a
  stamp, Measure objects). Measure objects was hidden behind "View more" (pdfcraft fork patch).
  The drawer now opens on that list instead of inside Add comments.

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
- ComPDF-compatible XFDF import and export (author in `title`), exact comment counts, replies.
- Owner-password-protected AES-256 PDFs: permissions reported, `unlock(ownerPassword)`.
- Locked view mode and comments-only annotate mode (pdfcraft UI profile patch).
- White-label branding from `brand/` (pdfcraft Brand patch); no ArtCraft or PdfCraft marks.
- Build (`scripts/build.ps1`), upstream update (`scripts/update-upstream.ps1`) and release
  (`scripts/release.ps1`) scripts; CI; Playwright e2e tests; docs.

### Based on

- pdfcraft upstream `6fb6b7cf6fd8dc105fdd56a764d207cff6140694` (0.10.0) with the
  `free-pdf-sdk/patches` commits (exact commit in `version.json`).

[Unreleased]: https://github.com/GeorgeMilroy/free-pdf-sdk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/GeorgeMilroy/free-pdf-sdk/releases/tag/v0.1.0
