# CI failures to fix (GitHub Actions on master)

Unit for workers. Edit any source file needed; no cargo; rustfmt touched files.

- Spelling (crate-ci/typos v1.52.2): fix every flagged word outside the typos.toml extend-exclude paths (words seen: branche, outsid, nd as a word). Grep comments in crates/**/*.rs and docs/*.md; fix the word, or add to typos.toml [default.extend-words] only for vanilla identifiers.
- Cargo lints and release lints: scan files in the last 20 commits (git log -20 --stat) for obvious clippy pedantic violations from AGENTS.md and fix them.
- Nix flake: check flake.nix and Cargo.lock agree; report only.

## Result
(record FIXED / STILL OPEN here)
