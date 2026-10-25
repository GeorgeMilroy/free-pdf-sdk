---
name: release-pdfcraft-update
description: End to end, unattended - take the latest pdfcraft upstream into Free PDF SDK, verify it, have independent reviewers check it, commit, bump the patch version, tag, push (fork first), wait for CI, publish to npm, update and deploy the landing site. Commits carry no Claude co-author line.
---

# Release a pdfcraft update

One command for the whole cycle the user used to do by hand (compare commits `7031604`
"Update pdfcraft to 0.13.0" and `70354bb` "Prepare 0.2.2"): update -> check -> commit ->
version -> tag -> push -> CI -> npm -> site. Run it start to finish without asking, except at
the stop points below.

Read first: `.claude/skills/update-pdfcraft/SKILL.md`, `docs/updating-upstream.md`,
`scripts/update-upstream.ps1`, `scripts/release.ps1`. This skill runs `/update-pdfcraft`'s
steps 1-4 as written, then replaces its step 5 ("never commit, never push") with the release
below. Every other rule of that skill still applies.

## Hard rules

- **No co-author or AI attribution anywhere.** Commit messages, tag messages and anything
  pushed must not contain `Co-Authored-By`, "Claude", "Generated with" or similar lines. This
  overrides any harness attribution reminder. Author/committer is the repository's own git
  config; never pass `--author` or change git config.
- Never `--no-verify`, never `git push --force` (only the `--force-with-lease=<branch>:<sha>`
  the update script prints for the fork's patch branch), never rewrite `main` or an existing tag.
- Never weaken, skip or delete a test or check to get green. `-SkipE2e` / `-SkipPdfcraftTests`
  are not allowed.
- Never print or read npm/firebase tokens or credentials.
- Version: a **patch** bump (`0.2.2` -> `0.2.3`). This command never exposes a new public
  feature id, mode preset or protocol change; new upstream commands go under existing features.

## Stop points (stop, report, do not push anything further)

- Nothing new upstream (`OLD..upstream/main` empty): report "already up to date" and stop.
- A rebase conflict you cannot resolve with upstream's code plus the patch's hook, a patch that
  would have to be weakened, or a test that fails after a genuine attempt to fix the cause.
- `/update-pdfcraft` step 3 says "ask the user" (a genuinely new feature, an API/protocol
  change, a new non-MIT/Apache licence, installing a new `wasm-bindgen` CLI).
- A reviewer finding that survives verification (step 3) and you cannot fix cleanly.
- npm publish (step 8.2) needs the user: the account signs in with a passkey, which npm only
  offers in an interactive terminal, and this shell isn't one (it fails with `EOTP`). Don't
  retry and don't ask for a code. Everything up to and including the git pushes is done; tell
  the user to run `npm run publish:npm:prod` in their own terminal (it prints a link, they
  approve with the passkey in the browser) and to say when it's done - then continue at 8.2's
  check. Also stop here if `npm whoami` fails: they run `npm login` first.
- CI red on the pushed commit.

When stopping before step 6, leave local work in place (commits included) and say exactly what is
done and what isn't. Nothing is pushed before step 6.

## 1. Preconditions

- This repo is clean, on `main`, and `git fetch origin` shows `main` == `origin/main`.
- `npm whoami` succeeds (check now so a long run doesn't end at a 401; if it fails, still do
  steps 2-7 and stop before 8).
- `firebase` CLI available (`npx firebase --version` or `firebase --version`).
- Let `PREV` = `js/package.json` version, `NEXT` = PREV with the patch number + 1; tag
  `vNEXT` must not exist locally or on origin, and `npm view @mecorelabs/free-pdf-sdk versions`
  must not list `NEXT`.

## 2. Update (from /update-pdfcraft, steps 1-4)

Run them exactly: the look-first summary (write it to the user as a progress note, don't wait
for an answer), `./scripts/update-upstream.ps1`, conflict resolution, placing new upstream
commands, then step 4's verification (`npm run build:sdk:local-fork`, `npm run test:e2e`,
screenshots compared, site images regenerated if the UI changed, wasm size compared,
`third-party/` and licences). Add the CHANGELOG `## [Unreleased]` entry in the style of the
0.2.2 section (summary line, Fixed / Added / Changed, "Based on" naming the upstream commit and
version). Write it for SDK users: what changed for them, not the patch mechanics.

### 2b. Landing site

The site (`site/`, see `site/README.md`) takes the version, the changelog and the feature/mode
tables from the repo on its own; the rest is written by hand and must still be true after the
update. With the SDK built (`dist/free-pdf-sdk-<version>/`):

1. Read `site/src/pages/home.html` and the docs in `site/src/docs/` against the CHANGELOG entry
   and the upstream log. Fix any claim the update made wrong (behaviour that changed, a feature
   now hidden or removed, a limitation that's gone). Add a user-visible capability that came in
   to the "More features" list or the docs page it belongs on, only if the web build actually
   offers it (check it in the demo). Keep the home page's style; don't rewrite sections that
   are still right.
2. Counts: modes and feature switches are filled in from `docs/reference/config.json`
   (`{{MODES}}`, `{{FEATURES}}`), and the build fails when `README.md` / `js/README.md` say
   otherwise - fix the READMEs, never the check. The language count ("23 languages", home page
   and `README.md`) is by hand: compare it with the locales pdfcraft ships.
3. If the UI changed, regenerate the images: `node site/build.mjs`, serve
   (`node scripts/serve.mjs --root site/public`), `node site/make-images.mjs`, build again.
   Look at the new images; a broken render is a failing check.
4. `node site/build.mjs` must pass (it checks links and front matter). Serve `site/public` and
   with Playwright load `/`, `/demo/` and `/docs/changelog/`: the version pill says `v<version>`,
   the demo's viewer opens the sample with no console errors, the changelog has the new entry.
5. These changes go in the update commit (step 4); say what changed on the site in its body.

## 3. Independent review (Workflow)

Before committing, run a Workflow (this skill authorises it). The reviewers get the diff and the
facts, not your conclusions. Give each the upstream range, the fork's
`git range-diff OLD_UPSTREAM..OLD_PATCH upstream/main..NEW_PATCH` in
`D:\Developments\storytold\pdfcraft`, and `git diff` here. Dimensions, one agent each:

1. **Patch stack** - every `free-pdf-sdk:` patch is still present (or its drop is justified
   because upstream does the same), each hook's default still means "pdfcraft as it ships", no
   patch weakened, conflict resolutions keep upstream's new code.
2. **Feature placement / web build** - every new upstream command, group, edit and annotation
   kind is placed under the right existing feature; nothing needing the filesystem, `rfd`,
   threads, OCR or signing is advertised in the web build.
3. **SDK surface** - `crates/viewer/src/host.rs` / `crates/core` adaptations don't change the
   RPC protocol or the JS API; `docs/reference/config.json` is unchanged or regenerated.
4. **Pins and release consistency** - Cargo.toml revs, Cargo.lock, NOTICE, `[patch.crates-io]`
   mirrors pdfcraft's, `third-party/` regenerated and licences OK, CHANGELOG entry accurate
   against the upstream log (nothing claimed that didn't come in, notable fixes not missing).
5. **Landing site** - `site/src/pages/home.html`, `site/src/docs/*.md` and the READMEs against
   the CHANGELOG entry and the web build: no claim the update made false, nothing advertised
   that the web build doesn't offer, regenerated images show the real viewer.

Each finding then gets two independent skeptics told to refute it against the code (kill on
majority refute). Fix confirmed findings, re-run the affected checks, and re-run the review once
for the changed area. A confirmed finding you can't fix is a stop point.

## 4. Commit the update

Stage only the update's files (`git status` first; nothing unrelated). One commit, subject
`Update pdfcraft to <version> (upstream <8-char sha>)` (or `Update pdfcraft to upstream <sha>`
when the version didn't change), body like `7031604`'s: the new pins and tag, conflicts and how
they were resolved, patches dropped/changed, where new commands were placed, wasm size change.
Use a message file or heredoc; then `git log -1 --format=%B` and confirm there is no trailer.

## 5. Prepare NEXT

Like `70354bb`. Bump `PREV` -> `NEXT` in: `Cargo.toml` (`[workspace.package] version` and the
`fps-core` / `fps-brand` path-dep versions), `Cargo.lock` (`cargo update -w --offline` or the
fps-* entries), `js/package.json`, `js/package-lock.json` (root `version` and
`packages[""].version`), `SDK_VERSION` in `js/src/free-pdf-sdk.ts`, the check in
`e2e/sdk.spec.ts`. CHANGELOG: rename `## [Unreleased]` to `## [NEXT] - <today>` and put a fresh
empty `## [Unreleased]` above it. `grep -rn PREV` (excluding CHANGELOG history, lockfile
dependencies of other packages, `dist/`, `target/`, `node_modules/`) must find nothing left.
Commit `Prepare NEXT` with a body like `70354bb`'s. No trailer.

## 6. Push the fork, release, push this repo

1. Fork first - run the push commands `update-upstream.ps1` printed, in its order (patch branch
   with its `--force-with-lease`, the `fps-upstream-*` tag, `main`). If the lease is rejected,
   stop: someone else moved the branch.
2. `./scripts/release.ps1 -Version NEXT` **without** `-LocalFork`: this proves the pushed fork
   commit fetches from GitHub as CI will, runs every check, builds `dist/` and the zip, and
   tags `vNEXT`. A failure here: fix, amend nothing that's pushed, re-run.
3. `git push origin main` then `git push origin vNEXT`.

Before each push: `git log origin/main..main --format=%B` (and the tag message) contains no
`Co-Authored-By` / Claude line.

## 7. Wait for CI

Watch the CI run for the pushed `main` commit with the Monitor tool (public API, no auth):
`https://api.github.com/repos/GeorgeMilroy/free-pdf-sdk/actions/runs?head_sha=<sha>` until
`status` is `completed`. `conclusion` must be `success`; otherwise read the failed job's log
(`.../actions/runs/<id>/jobs`), report, and stop - don't publish.

## 8. Publish and deploy

1. `dist/npm/package.json` version is NEXT; `npm run check:npm-package:dev` (dry run) - look at
   the file list: `dist/`, `bin/`, `viewer/`, licences, NOTICE, `third-party/`, nothing stray.
2. The user runs `npm run publish:npm:prod` (`--auth-type=web`, passkey in the browser) in
   their own terminal - see the stop point. Then `npm view @mecorelabs/free-pdf-sdk version`
   == NEXT.
3. Only after npm has NEXT (the site's install instructions point at it):
   `npm run deploy:hosting:prod` (builds the site from `dist/free-pdf-sdk-NEXT/`, deploys to
   the `free-pdf-sdk` Firebase project). `firebase` not logged in is a stop point like npm's:
   tell the user to run `firebase login`, rerun resumes here.
4. Check the live site, `https://freepdfsdk.mecorelabs.com`: `/sdk/version.json` has
   `"sdk": "NEXT"`, the home page's pill says `vNEXT`, `/docs/changelog/` has the NEXT entry,
   and with Playwright `/demo/` opens the sample with no console errors. Firebase can take a
   minute to serve the new release; retry a few times before calling it a failure.

## 9. Report

Upstream range; fixes and features that came in; conflicts and resolutions; patches
dropped/changed; feature placement; review findings and their fixes; wasm size change; the two
commit shas; tag; CI run link and result; npm version; what changed on the landing site and the
live site check. Anything skipped, and why.

## Resuming

Run again after a stop: detect where it got to instead of starting over - an
`Update pdfcraft` commit already on `main` skips 2-4, a `Prepare NEXT` commit skips 5, tag
`vNEXT` on origin skips 6, a green CI run for `origin/main` skips 7, NEXT already on npm skips
8.1-8.2, the live site serving NEXT skips 8.3. Re-check the preconditions that still apply.
