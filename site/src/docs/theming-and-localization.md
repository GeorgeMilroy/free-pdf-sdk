---
title: Theming and localization
description: Match the viewer to your app - light, dark or system theme, interface language with fallback, comment author, and white-label branding when you build the SDK yourself.
section: Guides
order: 52
---

# Theming and localization

The viewer draws its own interface, so your page's CSS doesn't reach it. You set its theme,
language and comment author through options and methods, and you can rebrand it completely by
building the SDK with your own logos and colours.

## Choose a theme

`theme` is `"light"`, `"dark"` or `"system"` (the default). `system` follows the browser's
`prefers-color-scheme` preference.

```ts
const viewer = await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  theme: "dark",
  document: { url: "/files/brochure.pdf" },
});
```

Change it at any time with `setTheme`:

```ts
const toggle = document.querySelector<HTMLInputElement>("#dark-mode")!;
toggle.addEventListener("change", () => {
  void viewer.setTheme(toggle.checked ? "dark" : "light");
});
```

If your app has its own theme switch, pass its value instead of relying on `system`, so the
viewer and the page always agree. With a framework component, bind the prop and the component
calls `setTheme` for you:

```tsx
<FreePdfViewer baseUrl="/free-pdf-sdk/" theme={isDark ? "dark" : "light"} document={{ url }} style={{ height: "80vh" }} />
```

```vue
<FreePdfViewer base-url="/free-pdf-sdk/" :theme="isDark ? 'dark' : 'light'" :document="{ url }" style="height: 80vh" />
```

Users can also change the theme in the viewer's preferences. To keep it under your control, turn
off the `preferences` feature, which removes the preferences dialog and theme switching:

```ts
await FreePdf.init(el, { baseUrl: "/free-pdf-sdk/", theme: "light", features: { preferences: false } });
```

An invalid value rejects with `BAD_REQUEST`.

## Set the interface language

`locale` takes a language tag such as `en`, `de`, `fr-CA` or `zh-Hans`. The viewer uses the
closest language it has:

1. the full tag (lower-cased, `_` read as `-`), for example `pt-br`;
2. otherwise the primary language, `pt`;
3. otherwise English, `en`.

```ts
const viewer = await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  locale: navigator.language, // for example "de-DE"
});
```

`setLocale` changes the language later and resolves to the language actually used, so you can
tell whether a fallback happened:

```ts
const used = await viewer.setLocale("pt-BR");
if (used === "en") console.info("Portuguese is not available; the viewer uses English.");
```

If you don't set `locale`, the viewer chooses the language itself. A tag must contain only
letters, digits, `-` or `_` (otherwise `BAD_REQUEST`) and be at most 35 characters (otherwise
`TOO_LARGE`).

Error messages in `FreePdfError.message` are always English and meant for logs; translate the
`code` for your users.

## Set the comment author

New comments are signed with an author name, which appears in the comment list and is exported
in the XFDF `title` attribute. Pass the signed-in user's name:

```ts
const viewer = await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  mode: "review",
  author: currentUser.displayName,
});

// When the user changes, or after they edit their profile:
await viewer.setAuthor(currentUser.displayName);
```

The `author` you pass wins over a name the user saved in the viewer's own settings, and it is
kept when another document is opened. Control characters are removed and surrounding spaces
trimmed; an empty name rejects with `BAD_REQUEST`, and names longer than 200 characters with
`TOO_LARGE`. Without an `author`, the viewer uses the name saved in its settings, or `Guest`.

## Loading animation

While the viewer starts, and while a document opens, a loading animation covers it. The
default is a small grey spinner on the viewer's background colour. Replace it with your own
SVG or GIF through `loader`:

```js
// An image URL: SVG, GIF, animated WebP or PNG, or a data: URL.
await FreePdf.init(el, { baseUrl: "/free-pdf-sdk/", loader: "/img/loading.gif" });

// Or with options.
await FreePdf.init(el, {
  baseUrl: "/free-pdf-sdk/",
  loader: {
    svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 50 50">…</svg>',
    size: "64px",          // width as a CSS length (default 48px); height keeps the aspect ratio
    background: "#ffffff", // CSS colour behind it (default: the viewer's background for the theme)
    label: "Opening your file", // announced to screen readers (default "Loading")
  },
});

// No loader at all.
await FreePdf.init(el, { baseUrl: "/free-pdf-sdk/", loader: false });
```

Give `src` (a URL) or `svg` (markup); with neither, the built-in spinner is used. Inline SVG is
shown as an image, so scripts in it never run and it can't load other files: animate it with
CSS or SMIL inside the SVG. It may be up to 1 MiB (`TOO_LARGE` otherwise), and a value that
isn't a string rejects `init` with `BAD_REQUEST`. The loader is read once, when the viewer
mounts. In the Web Component, use the `loader` attribute with a URL, or `loader="none"`.

## White-label branding

The viewer shows a product name, logos and an accent colour (in the About dialog and the
interface). They come from the `brand/` folder of the SDK's source repository and
are compiled into the WebAssembly module, so changing them means building the SDK yourself.
Nothing about branding can be changed at runtime.

| File in `brand/` | What it sets |
|---|---|
| `brand.toml` | Product name, accent colours, About text and link, the settings storage key |
| `logo-light.svg` | Logo on light backgrounds |
| `logo-dark.svg` | Logo on dark backgrounds |
| `mark.svg` | The square mark (About dialog) |

`brand.toml`:

```toml
# One key = "quoted value" per line. Comments go on their own line, never after a value.
app_name = "Acme Proofing"
# Accent on the light theme, then on the dark theme: "#RRGGBB".
accent_light = "#0B5FFF"
accent_dark = "#6EA2FF"
about_text = "Acme Proofing viewer. Built on open-source software: see NOTICE and third-party/ in the SDK folder."
about_url = "https://acme.example/proofing"
storage_key = "acme-proofing"
```

- **`app_name`**: replaces the product name in the interface. Empty means "Free PDF SDK".
- **`accent_light`, `accent_dark`**: `#RRGGBB`. If you give only one, it is used for both
  themes; with neither, the viewer keeps its default accent. Choose colours with enough contrast
  against white and dark grey text backgrounds.
- **`about_text`, `about_url`**: shown in the About dialog. The URL must start with `https://`,
  or it is ignored.
- **`storage_key`**: the key under which the viewer saves its preferences in the browser. Use
  your own so your viewer's settings don't mix with another Free PDF SDK app on the same origin.

Replace the SVGs with your own, keeping the file names. They must be self-contained: no external
images, fonts or scripts. Convert text to outlines so the logo renders the same everywhere.

Then build. The build script needs the Rust toolchain with the `wasm32-unknown-unknown` target,
`wasm-bindgen-cli` at the version locked in `Cargo.lock`, and Node.js with `npm install` run in the
repository root (for `wasm-opt`); it stops with a message when one is missing.

```bash
pwsh scripts/build.ps1 -Zip
```

The build writes `dist/free-pdf-sdk-<version>/` (the folder to serve, with the wrapper) and
`dist/npm/` (the npm package layout, including `copy-viewer`), and with `-Zip` a release zip.
Deploy that output instead of the published package, and keep your branded build's wrapper and
viewer together: the wrapper warns in the console when the two versions differ.

### Licences when you rebrand

Free PDF SDK is licensed MIT OR Apache-2.0, so you may ship it under your own name. Keep the
`NOTICE`, `LICENSE-MIT`, `LICENSE-APACHE` and `third-party/` files with the viewer: they hold the
licences of the open-source components the viewer embeds. The default About text points to
them; if you replace it, keep a pointer to those notices somewhere your users can find it.
