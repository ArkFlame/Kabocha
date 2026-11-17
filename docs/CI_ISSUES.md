# CI issues log

Observed on origin/master 890198966 (GitHub Actions, 2026-11-17). Checked with `gh run list`, `gh run view --job <id> --log`.

| # | Workflow / job | Failure | Root cause | Fix |
|---|---|---|---|---|
| 1 | Spelling (typos v1.52.2) | `clea`, `issu` in docs/PROCESS.md; `dbe`, `daa`, `ede`, `onl`, `isssue`, `placment`, `incorect`, `interract`, `attemping`, `strangly`, `replacable`, `disconnec` in docs/GAPS_FIXED.md | PROCESS.md script descriptions were cut mid-word; GAPS_FIXED.md is generated and quotes upstream issue titles and commit hashes verbatim | Completed the two words; added docs/GAPS_FIXED.md to `typos.toml` extend-exclude |
| 2 | Cargo: Check formatting | rustfmt diffs in bot.rs, configured_feature.rs, player_auth_input.rs, boat.rs, entity/mod.rs, cushion.rs, datapack/mod.rs (2), locate.rs (2) | Workers run rustfmt on single files only | `cargo fmt` |
| 3 | Cargo: Run lints (clippy affected packages) | `dead_code` `BoatEntity::position_rider`; `must_use_candidate` `is_embedded_pack`; `question_mark` x2 in fishing_bobber.rs | Worker commits were never clippy-checked | Removed the dead method, added `#[must_use]`, used `?` |
| 4 | Cargo: release lints | Same errors as 3 (release clippy builds pumpkin-core) | Same | Same |
| 5 | docker.yml | "workflow file issue" (run has no jobs) | A duplicate `build-args` key under the build-push step in our fork edit | Removed the duplicate key |
| 6 | Versioning | skipped | Not a failure | None |

Still unreproduced locally: nextest, gametest, check_wit, android, cargo deny. Tracked in docs/ENDGAME.md.

## Master run 44161349921 (commit ee1389896, 2026-11-17)

Failing jobs: 2 of 8 runs checked (Cargo Build, Test, and Linting: `Run lints` and `Run release lints`). Both fail on the same 9 clippy errors in pumpkin-world (`-Dwarnings`). Other runs: Spelling success, Versioning skipped, Cargo run 44161221999 cancelled.

| # | Job | Error (file:line) | Root cause | Fix | In tree at ee1389896? |
|---|---|---|---|---|---|
| 7 | Run lints + Run release lints | unknown lint `clippy::large_unopt_stack_frames` (pumpkin-data `configured_features_generated.rs:6`, emitted by `tools/pumpkin-codegen/src/configured_feature.rs:146`) | Lint name does not exist in the CI stable clippy | Renamed to `clippy::large_stack_frames` in codegen and generated file | Yes, fixed in worktree |
| 8 | Run lints + Run release lints | `return` unneeded (pumpkin-world `chunk_system/generation_cache.rs:565`, `:662`) | Trailing `return;` in last match arm | Removed `return;` | Yes, fixed in worktree |
| 9 | Run lints + Run release lints | missing `;` (`chunk_system/schedule.rs:674`) | `semicolon_outside_block` (workspace deny) | Added `;` | Yes, fixed in worktree |
| 10 | Run lints + Run release lints | collapsible `else { if let }` (`chunk_system/schedule.rs:1642`) | Nested if-let inside else | Flattened to `else if let` | Yes, fixed in worktree |
| 11 | Run lints + Run release lints | `large_stack_frames` 921885 bytes (`generation/surface/mod.rs:270`, plus `configured_features_generated.rs:9`) | Inlined neighborhood biome lookup | `#[expect(clippy::large_stack_frames, reason)]` on `get_biome`; generated file covered by item 7 rename | Yes, fixed in worktree (expect on `get_biome`) |
| 12 | Run lints + Run release lints | `match` single pattern and `option_if_let_else` (`generation/surface/mod.rs:270`) | Some/None match | `map_or_else` | Yes, fixed in worktree |
| 13 | Run lints + Run release lints | unfulfilled `#[expect(clippy::panic)]` (`generation/surface/mod.rs:259`) | No panic left in `get_biome` | Replaced that expect with item 11 | Yes, fixed in worktree |
| 14 | docker, nix, nightly, release, ferrflow, sync-wit | not failing in run 44161349921 | n/a | None | n/a |

Not verified by compile in this pass (workers do not run cargo). Orchestrator: run `scripts/integrate.sh` and `cargo run --locked -p pumpkin-codegen` to confirm generated output matches the codegen edit. Note: local clippy on the tree did not reproduce items 8-13; the CI stable toolchain flags them, so treat the local clean result as unreliable for pumpkin-world.
