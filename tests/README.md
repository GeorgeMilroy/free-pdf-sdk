# tests/

Rust integration tests and synthetic fixtures (crate `fps-tests`).

| File | What |
|---|---|
| `lib.rs` | Fixture generators: `plain_pdf(pages)` and `protected_owner_only(pages)` (AES-256, owner password `fps-owner-secret`, typical owner-only permissions). Deterministic. |
| `make_fixtures.rs` | `cargo run -p fps-tests --bin make-fixtures` writes the PDFs to `fixtures/` for the e2e tests. |
| `integration.rs` | XFDF round trips, the unlock flow, replace/delete semantics, known losses. |
| `fixtures/*.xfdf` | Hand-written in the shape web PDF viewers commonly export. Synthetic: no customer data. |
| `fixtures/real/` | Optional, git-ignored: put XFDF exported from production here to run `real_xfdf_samples_round_trip` against it. Never commit these. |

Run: `cargo test -p fps-tests -j 2` (with `. scripts/dev-env.ps1 -LocalFork <pdfcraft clone>`
while the fork commit is unpushed).
