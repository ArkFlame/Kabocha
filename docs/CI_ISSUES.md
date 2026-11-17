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
