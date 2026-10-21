# site/

The Free PDF SDK website: the landing page, the live demo (the real SDK, served from `/sdk/`)
and the documentation, for people (HTML) and for AI agents (`/llms.txt`, `/llms-full.txt`,
`/docs/<page>.md`, `/schema/viewer-options.schema.json`). It deploys to Firebase Hosting.

## Build

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1   # the SDK, into dist/free-pdf-sdk-<version>/
node site/build.mjs                                           # the site, into site/public/
node scripts/serve.mjs --root site/public                     # look at it on http://127.0.0.1:8780/
```

`site/build.mjs` fails on a broken internal link, a docs page without front matter, or a missing
SDK build. Set `SITE_URL` to build for another domain (canonical links, sitemap, llms.txt);
the default is `url` in `site.config.json`.

## Deploy (Firebase Hosting)

1. Install the Firebase CLI (`npm install -g firebase-tools`) and sign in (`firebase login`).
2. The project is `free-pdf-sdk` (`site/.firebaserc`); the address is `url` in `site.config.json`.
3. From the repository root:
   - `npm run deploy:hosting:prod` builds the site from the current SDK build and deploys it live.
   - `npm run deploy:sdk-and-hosting:prod` rebuilds the SDK first (`build:sdk`), then does the same.
   - `npm run preview:site:dev` serves the built site locally first, if you want a look.
   - Until the pdfcraft fork is pushed, build the SDK with `npm run build:sdk:local-fork`.

`firebase.json` serves `.wasm` as `application/wasm`, caches the hashed viewer files for a year
(their names change with every build) and never caches `viewer/index.html`, so a new SDK
version reaches visitors on their next visit. Firebase compresses responses (gzip/brotli)
automatically.

## Content

| Path | What it is |
|---|---|
| `src/docs/*.md` | Documentation pages (see `src/docs/README.md` for the conventions). |
| `src/pages/home.html`, `src/pages/demo.html` | The landing page and the live demo. |
| `src/assets/` | CSS, JavaScript and images (`img/` holds the hero screenshot and icons). |
| `site.config.json` | Site name, tagline, description, URL, optional source code link. |

Tables of modes, features and options in the docs come from `docs/reference/config.json`, which
`cargo test -p fps-core` keeps in step with the code.
