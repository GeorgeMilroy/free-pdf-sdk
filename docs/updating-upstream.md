# Updating pdfcraft

Free PDF SDK uses pdfcraft as git dependencies pinned to one commit, plus a short stack of
patches on the fork branch `free-pdf-sdk/patches`. Taking a new upstream means rebasing that
stack and moving the pins.

## The pieces

| Where | What |
|---|---|
| `D:\Developments\storytold\pdfcraft` | Your clone. `origin` = github.com/GeorgeMilroy/pdfcraft (fork), `upstream` = github.com/storytold/pdfcraft. |
| `D:\Developments\storytold\pdfcraft-fps` | A worktree of `free-pdf-sdk/patches`, so your `dev` checkout is never disturbed. |
| `Cargo.toml` `[workspace.dependencies]` | `git = "https://github.com/storytold/pdfcraft", rev = "<upstream commit>"` |
| `Cargo.toml` `[patch."https://github.com/storytold/pdfcraft"]` and `[patch.crates-io]` | `git = "https://github.com/GeorgeMilroy/pdfcraft", rev = "<patch-branch commit>"` |
| tags `fps-upstream-<date>` | Every patch-branch commit the SDK was built from, so it stays reachable after the branch is force-pushed. |

## The patches

```
git -C D:\Developments\storytold\pdfcraft log --oneline upstream/main..free-pdf-sdk/patches
```

Each is one concern, prefixed `free-pdf-sdk:`, an opt-in hook whose default keeps pdfcraft's
behaviour, with its own tests in `crates/ui-egui/tests/`. They follow pdfcraft's CLAUDE.md:
never panic, no `unsafe`, checked arithmetic, scoped `cargo test -p <crate> -j 2` (never a full
`--workspace` build). Each can be offered upstream as a PR; once upstream has an equivalent,
drop the patch in the next rebase.

## The update, step by step

`scripts/update-upstream.ps1` does all of this and stops at the first failure. It never pushes.

1. `git fetch upstream` (and `origin`).
2. Bring the fork's `main` up to `upstream/main`: fast-forward when possible; when the fork has
   its own commits (it currently carries a dependabot bump), merge `upstream/main` into it
   locally. A conflict stops the script.
3. Rebase `free-pdf-sdk/patches` onto `upstream/main` in its worktree. On a conflict: resolve in
   the worktree (`git status`, edit, `git add`, `git rebase --continue`), run the script again.
4. Scoped pdfcraft checks for the patched crates: `cargo fmt -p … --check`,
   `cargo clippy -p … -j 2 --all-targets -D warnings`, `cargo test -p … -j 2`.
5. Tag the new patch commit `fps-upstream-<yyyymmdd>`.
6. Move the pins in `Cargo.toml` (upstream commit and patch commit) and the commit in `NOTICE`.
7. `cargo update -p pdfcraft-engine -p pdfcraft-ui-egui`, regenerate `third-party/`, then this
   repository's fmt, clippy (native and wasm), tests, build and e2e tests.

Then it prints the push commands, for example:

```powershell
git -C "D:\Developments\storytold\pdfcraft" push --force-with-lease=free-pdf-sdk/patches:<old> origin free-pdf-sdk/patches
git -C "D:\Developments\storytold\pdfcraft" push origin fps-upstream-20261020
git -C "D:\Developments\storytold\pdfcraft" push origin main
git -C "D:\Developments\free-pdf-sdk" push origin HEAD
```

Push the fork first: CI (and anyone else building) fetches the patch commit from GitHub.

```powershell
./scripts/update-upstream.ps1                         # defaults to the paths above
./scripts/update-upstream.ps1 -PatchedCrates pdfcraft-ui-egui,pdfcraft-xfdf
```

## Building before the fork commit is pushed

Cargo fetches `https://github.com/GeorgeMilroy/pdfcraft` at the commit in `Cargo.lock`. Before
you push it, point Cargo at your clone for this shell only:

```powershell
. ./scripts/dev-env.ps1 -LocalFork D:\Developments\storytold\pdfcraft
cargo test -p fps-tests -j 2
./scripts/build.ps1 -LocalFork D:\Developments\storytold\pdfcraft
```

It sets `CARGO_NET_GIT_FETCH_WITH_CLI=true` and a process-scoped git `url.<clone>.insteadOf`
rewrite (`GIT_CONFIG_COUNT/KEY/VALUE`). Your global git config is untouched, and `Cargo.toml` and
`Cargo.lock` keep the real URL, so nothing changes after the push.

## Adding a patch

1. Check first that the SDK can't do it from outside: `crates/viewer/src/host.rs` wraps
   `PdfCraftApp` and can reach every public field and method between frames.
2. In the worktree: one commit, `free-pdf-sdk: <what>`, default = unchanged behaviour, a test
   that fails without it (revert the hook and run it), scoped fmt/clippy/test.
3. Move the patch pin in `Cargo.toml` to the new commit, `cargo update -p pdfcraft-ui-egui`,
   and run this repository's tests.
4. List it in docs/architecture.md.
