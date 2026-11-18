# CI failures to fix (GitHub Actions on master)

Unit for workers. Edit any source file needed; no cargo; rustfmt touched files.

- Spelling (crate-ci/typos v1.52.2): fix every flagged word outside the typos.toml extend-exclude paths (words seen: branch, outside, nd as a word). Grep comments in crates/**/*.rs and docs/*.md; fix the word, or add to typos.toml [default.extend-words] only for vanilla identifiers.
- Cargo lints and release lints: scan files in the last 20 commits (git log -20 --stat) for obvious clippy pedantic violations from AGENTS.md and fix them.
- Nix flake: check flake.nix and Cargo.lock agree; report only.

## Result

Third pass (shared checkout, all source files editable). No cargo/git/shell run by this worker; rustfmt is not needed (only a Python file and this doc were touched).

- Spelling (typos v1.52.2): FIXED. Word-boundary scan of every text file outside the typos.toml extend-exclude paths found zero remaining `branch` or `outside` words (already resolved by earlier passes; their only hits are in `crates/pumpkin-data/src/generated/` and in excluded `docs/tracker/`, `docs/investigations/`, `docs/GAPS_FIXED.md`). The sole `nd` word left in a non-excluded file was the format-width parameter `nd` in `scripts/model-eval.py:265`; renamed to `decimals` (all callers pass it positionally or omit it, so no behaviour change). No word was a vanilla identifier, so no `[default.extend-words]` entry was added.
- Cargo lints / release lints: VERIFIED (no code edit needed). Every clippy error recorded in `docs/CI_ISSUES.md` items 7-13 is already fixed in the current tree: `chunk_system/schedule.rs:674` has its trailing `;`, `schedule.rs:1642` is a flattened `else if let`, the `generation_cache.rs` stage match arms have no trailing `return`, `generation/surface/mod.rs:273` uses `map_or_else`, and the generated allow list (codegen `tools/pumpkin-codegen/src/configured_feature.rs:150` and `crates/pumpkin-data/src/generated/configured_features_generated.rs:2-11`) carries both the valid `clippy::large_stack_frames` and `unknown_lints`/`renamed_and_removed_lints`, which suppress the stale `large_unopt_stack_frames` name. A grep for the pedantic patterns named in AGENTS.md (`map_or(false/true, ..)`) found only pre-existing, unrelated test/API sites, none from the CI-documented set. UNVERIFIED: the literal `git log -20 --stat` scan could not be run by a worker (shell denied).
- Nix flake: REPORT only, agrees, no change. `flake.nix` reads the package version from the workspace manifest (lines 32, 46), takes `cargoLock.lockFile = ./Cargo.lock` (line 50), and pins the MC 26.3 server jar URL/hash (lines 38-41), matching `const MC_VERSION: &str = "26.3"` in `crates/pumpkin-data/build.rs:5`. `build.rs` resolves the jar URL from the version manifest at that version; the flake comment (lines 34-37) documents keeping the pin in step with it.

