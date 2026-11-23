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

## Audit 2026-11-18 (latest master runs 44224466341, 44210767724, 44207855376, 44138776888)

Full table: `docs/CI_AUDIT.md`.

| # | Job | Error | Class | Status |
|---|---|---|---|---|
| 15 | Run lints, Run release lints | `unknown lint: clippy::large_unopt_stack_frames` (`pumpkin-data/.../configured_features_generated.rs:6`, from `tools/pumpkin-codegen/src/configured_feature.rs:146`). Upstream uses `large_stack_frames` | code | Open for orchestrator: rename back, regenerate |
| 16 | Run lints, Run release lints | `large_stack_frames` 921885 bytes (`configured_features_generated.rs:9`) | code | Open, follows from 15 |
| 17 | Check formatting | rustfmt diffs in `crates/pumpkin-core/src/data/datapack/mod.rs` 1957, 1965, 3744 | code | Open: `rustfmt --edition 2024` on file |
| 18 | Validate Flake and Build | 122 compile errors in `pumpkin-core` (run 44210767724, older commit) | code | Re-check latest run 44224466320 |
| 19 | Sync WIT to mirror repo | empty `client-id` (no `BOT_APP_CLIENT_ID`) on fork | fork-env | Fixed: job gated to upstream repo |
| 20 | Release (sign, publish) | SignPath and release publish need upstream secrets | fork-env | Fixed: gated to upstream repo |
| 21 | Cargo test, gametest | never reached on fork (lints red) | blocked | Re-check after 15-17 |
| 22 | docker deploy-docker-image | startup failure with no jobs at sha 8901989 | infra or old sha | Current file gated and parses; re-check on next run |
| 23 | Nix magic-nix-cache | FlakeHub unauthenticated (non-fatal) | fork-env | Noted, not gated |
| 24 | WIT v0.2 `event.wit` | 836 lines removed vs upstream | plugin API, review | Open for orchestrator |

| 25 | Spelling | audit report emitted a misspelled plural header; generated vanilla tag name | code | Fixed: label map in autosync-audit.sh, allowlists in typos.toml; CI run 45171758770 success |
| 26 | Test (all platforms) | create_matches_direct_encode unwrap None: current writer emits mandatory color VarInt, translator read bool+optional | code | Fixed in 2f923ad66: read VarInt, normalize 21 to -1; proof test passes |
| 27 | Test (Linux ARM64) | woodland mansion beard assertion failed after codegen reverted hand-edited generated data | code | Fixed in eeb70ab08: codegen hardcodes beard_thin for mansion; proof test passes |
| 28 | GameTest and Bot Harness | sign editor wait was iteration-bounded and flaked under load | code | Fixed in dafc55d75: 5s wall-clock deadline; local full harness passed |
| 29 | all Cargo jobs | auto-audit docs pushes triggered cancel-in-progress, killing in-flight tests | ops | Fixed in d96a853de: audit commits use skip-ci; verified no runs on later audit commits |
| 30 | Test (Linux ARM64), Test (macOS), Test on Android AArch64 | runner shutdown signal exit 143, macOS cancelled, Android logs BlobNotFound or job stuck | infra | BLOCKED: reproduced on four separate runs including a fresh workflow_dispatch; not a code failure |
