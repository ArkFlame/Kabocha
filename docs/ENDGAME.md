# Endgame: what happens at 100%

Trigger: docs/LEDGER.tsv has no TODO/IN-PROGRESS/COMMITTED-UNLANDED/FAILED/BLOCKED rows, every gap row in TRACKER.md is LANDED or DONE (parity 100%), and every upstream issue and PR is closed or merged here. Only then start this phase. Until then, keep fixing.

## 1. Full compile and CI green (user-mandated 2026-11-17)
Last observed push CI on origin/master (GitHub Actions) was NOT green:
- Failing: `Check formatting`, `Run lints`, `Run release lints`, `Spell Check with Typos`.
- In progress: `Nix Flake / Validate Flake & Build`.
- Skipped because the checks above failed: build affected packages, Android AArch64 builds, release build, tests, GameTest and Bot Harness, versioning, canary release.
- Passing: `Plan CI`, `Validate pumpkin-plugin-wit`.

Required before declaring done, in this order, each fixed forward on a worker branch and landed:
1. `cargo fmt --all --check` clean.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` clean (debug), then the release lints job.
3. Typos clean (`typos`; fix spelling or extend the project typos config, never silence real misspellings).
4. `scripts/cargo.sh check --workspace --locked` and a full build pass; `cargo nextest` on every crate; GameTest and bot harness pass.
5. Nix flake builds; Android AArch64 build jobs compile.
6. Push and read the real CI result with `gh run list` / `gh run view`; do not claim green from local runs alone.

## 2. Code cleanup
- Remove dead code, unused imports and features, stale TODO/FIXME that are resolved, duplicate helpers left by integration merges.
- Delete worker leftovers: scratch files, per-issue notes, obsolete scripts that nothing calls.
- No new warnings; keep `unwrap`/`expect`/`panic` denials intact.
- Re-run the parity tooling after cleanup so nothing regressed.

## 3. GitHub cleanup
- Close every resolved issue and PR on the fork with a short note naming the landing commit.
- Delete merged `kab/*` branches locally and on origin (`scripts/clean-merged.sh`), drop old `refs/kab/*` and stale stashes, prune worktrees under `../Kabocha-wt`.
- Make master the only long-lived branch; tag the final state.
- Update README/docs/TRACKER.md/LEDGER to the final numbers; keep docs/SYNC.md current.

## 4. Stay done
Keep the hourly upstream sync; any new upstream commit or issue reopens the loop.
