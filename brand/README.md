# brand/

Placeholder branding for Free PDF SDK. Replace these files and keep their names:

| File | Used for |
|---|---|
| `logo-light.svg` | Wordmark on light backgrounds (About dialog) |
| `logo-dark.svg` | Wordmark on dark backgrounds (About dialog) |
| `mark.svg` | Square mark (About dialog, Home) |
| `brand.toml` | Product name, accent colours, About text and URL, settings storage key |

`crates/brand` embeds them at compile time, so rebuild after you change them (`scripts/build.ps1`).
SVGs must be self-contained: no external images, fonts or scripts. Text in an SVG is drawn
with the viewer's own fonts (Inter), so convert lettering to paths if the exact font matters.
