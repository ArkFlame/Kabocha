GOAL: Kabocha final state. Reach 100% parity, a green build and CI, a clean repo and full documentation. No PRs to Pumpkin-MC yet. Master only: ff-merge and push each verified slice at once.

RULES
- You are the orchestrator. Use Claude subagents only (no OpenCode, quota is out). Up to 10 in parallel, each in its own worktree (scripts/wt.sh). Workers never run cargo; you run one scripts/cargo.sh check per batch (scripts/int-batch.sh, scripts/land.sh).
- Never ask the user. Decide from vanilla, Paper, .vendor/ and docs, then record the precedent in the commit or docs.
- Follow AGENTS.md and CONTRIBUTING.md: conventional subject-only commits, no em dashes, no unwrap/expect/panic, tracing instead of println, never hand-edit generated data.

PHASE 1: CLOSE THE GAPS (docs/TODO_2.md, docs/tracker/*.md, TRACKER.md)
1. Tracker is 99-100% but "closed" means an audit note exists, not that the fix is done. Read every docs/tracker/*.md and docs/TODO_2.md. List each remaining gap with its size (S/M/L).
2. Fix all of them, starting with S and M, then L. Known ones: Vex AI (charge goal, random move, copy owner target, lifespan), chest boat piglin anger, the player.rs statistics list, kab/i4498 (Bedrock CraftRecipeAuto).
3. Each fix ships the smallest automated test that fails without it. Verify behaviour against decompiled vanilla in .vendor/ (run scripts/vendor.sh if missing). Unverified audit notes must be re-checked, then marked VERIFIED.
4. Run scripts/parity.sh. Upstream tracker rows must match docs/PARITY.tsv. Run scripts/ledger.sh. TODO, FAILED, BLOCKED and COMMITTED-UNLANDED must stay at 0.

PHASE 2: GREEN BUILD AND CI (docs/ENDGAME.md)
1. Read .github/workflows/* and reproduce every job locally: fmt, clippy -D warnings (all targets, all features), release lints, typos, cargo deny/audit if configured, nextest, pumpkin-gametest, docs build.
2. Fix every failure at its root. Do not silence lints; a justified #[expect(.., reason = "..")] is the only exception. Zero warnings.
3. The workspace must compile on stable with --locked. Run scripts/check.sh -f for each touched crate, then once the full gate: cargo check, clippy and nextest across the workspace. Boot with scripts/boot.sh and run the bot scenarios.
4. Keep the Pumpkin professional bar: rustfmt clean, stable public plugin API (additive WIT only, bump PLUGIN_API_VERSION on breaking changes), no secrets, license headers and files intact.

PHASE 3: TECHNICAL DEBT AND CLEANUP
1. Grep the source for TODO, FIXME, HACK, XXX, dead code, duplicated helpers (the Bedrock item_stack_request duplicates keep recurring: dedupe at the source), oversized functions, stale feature flags, unused dependencies (cargo machete) and magic numbers. Fix them, simplest correct change.
2. Delete obsolete scripts, temp files, stale docs and generated churn. Do not touch uncommitted work you did not make.
3. Remove ALL merged and stale kab/* branches and ../Kabocha-wt worktrees (scripts/clean-merged.sh, then git worktree prune). Verify with git cherry that nothing unlanded is deleted. Leave only master (plus upstream remote refs).

PHASE 4: DOCUMENTATION
1. docs/PROCESS.md: how the orchestration worked (workers, batches, integration, ledger, tracker, sync), key scripts and pitfalls (merge docs/LEARNINGS.md).
2. docs/GAPS_FIXED.md: every gap, issue and PR closed, with commit hash, vanilla reference and test. Source it from LEDGER.tsv, PARITY.tsv and docs/tracker.
3. Update README, docs/DONE.md, docs/TODO.md (empty or only deferred L items with reasons) and docs/SYNC.md. Final TRACKER.md must show 100%.

PHASE 5: FINISH
1. One last sync: scripts/sync-upstream.sh, merge if behind, one workspace check.
2. Final gate: ledger clean, tracker 100%, CI jobs green locally, git status clean, origin/master equals master, only master branch left.
3. Report: commit range, test counts, anything deliberately deferred.

DONE WHEN every line of Phase 5 holds. Do not stop earlier.
