---
name: update-pdfcraft
description: Bring a new pdfcraft upstream (bug fixes, features) into Free PDF SDK safely - review what changed, rebase the fork's patch stack, place new commands in the UI profile, move the pins, verify, and report. Never pushes.
---

# Update pdfcraft

Bring the latest `storytold/pdfcraft` into Free PDF SDK. The mechanics are in
`docs/updating-upstream.md` and `scripts/update-upstream.ps1`; read both first. This skill adds
the judgement around them.

## Hard rules

- Never push, force-push, open upstream PRs or deploy. Print the push commands at the end.
- Never commit in this repository; the user commits. Committing in the fork worktree
  (`D:\Developments\storytold\pdfcraft-fps`) is fine, as part of the rebase.
- Follow pdfcraft's CLAUDE.md inside the fork: never panic, no `unsafe`, checked arithmetic,
  scoped `cargo test -p <crate> -j 2`, never a `--workspace` build.
- Never drop or weaken a `free-pdf-sdk:` patch silently. Never delete or loosen a test to make it
  pass. If something can't be resolved cleanly, stop and explain.
- Ask before installing anything other than Rust/npm dev tooling.

## 1. Look before changing anything

In `D:\Developments\storytold\pdfcraft`: `git fetch upstream --tags`, then let
`OLD` = the upstream rev in this repo's `Cargo.toml` `[workspace.dependencies]`.

- `git log --oneline OLD..upstream/main` and `git diff --stat OLD upstream/main`; read upstream's
  CHANGELOG/release notes for the range.
- Sort the changes into: bug fixes, new user-facing features (new commands, tool groups, edits,
  annotation kinds, panels), changes to `PdfCraftApp`'s public API (used by
  `crates/viewer/src/host.rs`), toolchain/`wasm-bindgen`/eframe bumps, changes to pdfcraft's
  `[patch.crates-io]` list, new dependencies (licences), new assets or translations.
- Conflict hot spots: files changed both upstream and by the patch stack
  (`git diff --name-only $(git merge-base upstream/main free-pdf-sdk/patches) free-pdf-sdk/patches`
  intersected with the upstream range).
- Write this summary to the user before step 2.

## 2. Run the update

This repository must be clean (the script checks). Then `./scripts/update-upstream.ps1`.

On a rebase conflict, resolve it in the worktree so the result is upstream's new code plus the
patch's hook, with the hook's default still meaning "pdfcraft as it ships". Re-run each
conflicted patch's own tests. If upstream now does what a patch does, drop the patch and say so.
Then `git rebase --continue` and run the script again.

If pdfcraft's `[patch.crates-io]` list changed, mirror it in this repo's `Cargo.toml`.
If `wasm-bindgen` changed, the installed CLI must match `Cargo.lock` (`scripts/build.ps1` checks; ask before
installing the new CLI).

## 3. Place new upstream features

The fork's `crates/ui-egui/src/features.rs` matches exhaustively, so a new command, group, edit or
annotation kind fails to compile or fails `tests/commands.rs` until it is placed. For each:

- Put it under the existing feature it belongs to when one fits (a new markup tool -> the
  comments feature, and so on).
- Check it works in the browser build. Anything needing the filesystem, native dialogs (`rfd`),
  threads or external models (OCR, digital signing) must be hidden in the web build, not
  advertised.
- A genuinely new feature also needs the SDK side: `crates/core/src/features.rs` (and its mode
  presets), `FEATURES` in `js/src/free-pdf-sdk.ts`, `FPS_WRITE_REFERENCE=1 cargo test -p fps-core`
  to regenerate `docs/reference/config.json`, the site docs, README/CHANGELOG counts and the
  landing page. Ask the user before exposing a new feature publicly or adding it to a mode preset.

If `PdfCraftApp`'s API changed, adapt `crates/viewer/src/host.rs` and `crates/core` without
changing the RPC protocol; a protocol change needs the user's approval.

## 4. Verify

The script runs the pdfcraft checks, this repo's fmt/clippy/tests, the wasm build and e2e. Then:

- `npm run build:sdk:local-fork` (the fork commit isn't pushed yet), `npm run test:e2e`.
- Look at the screenshots (`e2e/screenshots/`, view/annotate x light/dark) and compare them with
  the previous ones; if the UI changed, regenerate the site images (`site/make-images.mjs`) and
  rebuild the site (`npm run build:site`).
- Compare the wasm size with the previous build and report the change.
- Regenerate `third-party/` if dependencies changed, and check new licences are MIT/Apache-
  compatible.

## 5. Report

Upstream range taken; bug fixes and features that came in; conflicts and how each was resolved;
patches dropped or changed; new features and where they were placed (and which need a decision);
API adaptations; size change; anything not done. Add a CHANGELOG "Unreleased" entry. End with
the push commands the script printed (fork first, then this repository), for the user to run.
