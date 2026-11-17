GOAL: Close every remaining Kabocha parity gap (about 700 STILL OPEN rows in docs/tracker/*.md and docs/TODO_2.md) with a continuously refilled pool of 40 parallel OpenCode workers editing ONE checkout, then compile once. Done when zero STILL OPEN rows remain except deferred ones with a reason (BLOCKED-DATA), the workspace builds, clippy -D warnings, typos and tests pass, TRACKER.md shows 100%, docs are current, git status is clean, origin/master == master, only master branch, no extra worktrees.

MODE: JUMP THE ROCKS. Workers never compile. Compile errors stack up and that is fine: most edits compile, so fix the few errors once at the end instead of stopping for each. No worktrees (they cost minutes each and cause merge conflicts). All workers edit the main checkout /run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha on master.

POOL: `KAB_ROOT=$PWD PATH=$HOME/.opencode/bin:$PATH setsid nohup python3 scripts/gap-pool.py 40 >/tmp/kabocha-runs/pool.log 2>&1 &` (run once, detached). One unit = one docs/tracker/*.md with STILL OPEN rows. A freed slot is refilled at once; two units naming the same .rs basename never run together (no edit collisions). Hard timeout 1200 s per worker, quota/rate-limit errors requeue the unit with backoff and stop the pool after 6 hits. Brief: docs/POOL_BRIEF.md. State: /tmp/kabocha-runs/pool/<unit>.{log,exit}; progress: tail pool.log. Model opencode-go/hy4-flash-free (verified; commandcode and deepseek quota is out until 2026-11-22, opencode/hy4-flash-free hit its free limit under 40 parallel). To resume after a limit resets, rerun the command: finished units (exit 0) are skipped. Do not poll in a loop: check pool.log only when notified or every few minutes.

WORKER RULES: edit only the source files named in the unit doc; no cargo, no git; verify against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/ (main repo only); no unwrap/expect/panic, tracing not println, no em dashes, never edit crates/pumpkin-data/src/generated; smallest pure unit test where a seam exists; rustfmt touched files; record FIXED/VERIFIED/STILL OPEN with a reason in a "## Result" section of the unit doc.

WHEN THE POOL IS DONE (POOL DONE in pool.log):
1. `git status`: review the diff size, drop obvious garbage (stray files, docs outside docs/tracker).
2. `cargo fmt --all`, then ONE `scripts/cargo.sh clippy --workspace --locked --all-targets --all-features --keep-going -- -D warnings`. Fix every error in one round (group by file; duplicated helpers from parallel workers are the usual cause), rerun until clean. Detached runs with setsid nohup; never modify the tree while cargo runs.
3. `scripts/cargo.sh test --workspace --locked --no-fail-fast`; rerun only failing crates. Known earlier failures are fixed on branch kab/h22: cherry-pick it first.
4. Cherry-pick unlanded worker branches (git cherry master kab/X): h1, h3-h9, h11-h19, h21-h27, h31, oq3, oq4, oq6; resolve conflicts; compile once more.
5. Commit in a few conventional subject-only commits (no body, no em dashes; hook adds the trailer), ff to master, `git push origin master`.
6. Regenerate: scripts/tracker-md.sh, ledger.sh, gaps-fixed.sh, parity.sh; update DONE.md, TODO.md (only deferred large items with reasons), GAPS_FIXED.md, README.
7. Phase 3 cleanup: scripts/clean-merged.sh, git worktree prune, remove ../Kabocha-wt and stale kab/* branches after `git cherry` shows nothing unlanded.
8. Hourly: scripts/sync-upstream.sh, append "YYYY-MM-DD HH:MM ahead A behind B" to docs/SYNC.md, push. If behind, merge upstream/master and compile once.
9. CI: verify `gh run list -R ArkFlame/Kabocha` is green (see docs/CI_ISSUES.md).

HARD RULES: never ask the user; decide from vanilla/Paper/.vendor and record precedent. Never run while-true loops or watchers except the bounded pool. Cargo only via scripts/cargo.sh, one run at a time, debug profile. Do not clobber uncommitted work you did not make. Keep user email and project details out of external requests.

OPEN WORK AFTER GAPS: claude-opencode-mcp install (subagent running at ../claude-opencode-mcp; once installed use its delegate_task/wait_any/list_runs tools instead of gap-pool.py).
