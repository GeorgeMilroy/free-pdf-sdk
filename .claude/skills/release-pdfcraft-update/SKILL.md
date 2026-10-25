---
name: release-pdfcraft-update
description: End to end, unattended - take the latest pdfcraft upstream into Free PDF SDK, verify it, have independent reviewers check it, commit, bump the patch version, tag, push (fork first), wait for CI, publish to npm and deploy the site. Commits carry no Claude co-author line.
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
- npm not logged in (`npm whoami` fails) or asks for a one-time password: everything up to
  and including the git pushes is done; tell the user to run `npm login` (or give the OTP) and
  run this command again - it resumes at step 8 (see "Resuming").
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
2. `npm run publish:npm:prod`. Then `npm view @mecorelabs/free-pdf-sdk version` == NEXT.
3. `npm run deploy:hosting:prod` (builds the site from `dist/free-pdf-sdk-NEXT/`, deploys to
   the `free-pdf-sdk` Firebase project).
4. Check `https://freepdfsdk.mecorelabs.com` serves NEXT (fetch the page / the SDK's
   `version.json` the site build references).

## 9. Report

Upstream range; fixes and features that came in; conflicts and resolutions; patches
dropped/changed; feature placement; review findings and their fixes; wasm size change; the two
commit shas; tag; CI run link and result; npm version; site check. Anything skipped, and why.

## Resuming

Run again after a stop: detect where it got to instead of starting over - an
`Update pdfcraft` commit already on `main` skips 2-4, a `Prepare NEXT` commit skips 5, tag
`vNEXT` on origin skips 6, a green CI run for `origin/main` skips 7, NEXT already on npm skips
8.1-8.2. Re-check the preconditions that still apply.
