# brand/

The Free PDF SDK brand. `crates/brand` embeds these files in the viewer at compile time, so
rebuild after changing them (`scripts/build.ps1`). The landing site (`site/`) uses the same files.

| File | Used for |
|---|---|
| `logo-light.svg` | Logo on light backgrounds (viewer About dialog, site header) |
| `logo-dark.svg` | Logo on dark backgrounds (white lettering) |
| `mark.svg` | The square "F" mark (About dialog, Home, favicons) |
| `brand.toml` | Product name, accent colours, About text and URL, settings storage key |
| `source/free-pdf-sdk-logo.png` | The original logo artwork the SVGs were traced from |

Brand colours (from the logo):

| Name | Hex | Use |
|---|---|---|
| Red | `#FF0801` | Mark, highlights, buttons on dark backgrounds |
| Deep red | `#AC000D` | Mark folds, hover states |
| Ink | `#080B0C` | Lettering, dark surfaces |
| Accent (light UI) | `#E10600` | Viewer accent on light theme (text contrast 5:1 on white) |
| Accent (dark UI) | `#FF4D40` | Viewer accent on dark theme |

SVGs must be self-contained: no external images, fonts or scripts. The logo lettering is
outlined (paths), so it renders the same everywhere.
