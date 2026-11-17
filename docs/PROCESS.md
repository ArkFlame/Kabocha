# Process

How Kabocha is run. The story and cost are in `docs/METHODOLOGY.md`; the worker rules are in `docs/WORKER.md`; capacity and build policy are in `docs/ORCHESTRATION.md` and `docs/BUILD.md`; pitfalls are in `docs/LEARNINGS.md`.

## Pipeline

1. **Queue.** Upstream issues, PRs and parity rows are mirrored into `docs/queue-*.tsv` and `docs/PARITY.tsv`.
2. **Brief and isolate.** One brief and one worktree (`scripts/wt.sh new <name>`, branch `kab/<name>`) per item.
3. **Implement.** A worker edits and commits in its worktree. Workers never run cargo, merge or push.
4. **Integrate.** `scripts/int-batch.sh` cherry-picks every unlanded branch into `kab/int` and runs one `cargo check --workspace --locked`. The orchestrator fixes compile errors there.
5. **Land.** Merge `kab/int` into master with a fast-forward or merge commit, push, then `scripts/clean-merged.sh`.
6. **Account.** `scripts/ledger.sh` (issues and PRs), `scripts/tracker-md.sh` (parity gaps), `scripts/gaps-fixed.sh` (what was fixed), `scripts/parity.sh` (diff against the upstream tracker).
7. **Sync.** `scripts/sync-upstream.sh` every hour; merge upstream in a `sync<N>` worktree when behind.

## States

- Ledger: TODO, IN-PROGRESS, COMMITTED-UNLANDED, DONE, SKIPPED, FAILED, BLOCKED.
- Tracker: LANDED, DONE, AUDITED, IN-FLIGHT, OPEN. AUDITED means an audit note exists in `docs/tracker/`, not that every gap in it is fixed.

## Quality gates

- `scripts/cargo.sh check --workspace --locked` before any landing.
- CI parity: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo clippy --release ...`, `typos`, `cargo nextest`, gametest. Use the stable toolchain with the `rustfmt` and `clippy` components.
- Conventional subject-only commits; `Co-Authored-By` trailer added by `scripts/hooks/commit-msg`.

## Pitfalls

- Integration cherry-picks, so `git branch --merged` lies. Use `git cherry master <branch>`.
- Duplicate Bedrock helpers (`populate_grid_from_auto_craft`, `take_from_main_inventory`) reappeared in many batches because independent workers fixed the same issue. Deduplicate at the source.
- Hundreds of worktrees fill the disk. Run `scripts/clean-merged.sh` after each landing.
- A syntax error in a generator script fails silently; check the generated file's counts after every edit.
- Kill workers by pid file, never with `pkill -f` combined with other work.
- Quotas end runs abruptly (OpenCode weekly limit). Workers must fail fast with an exit file so the orchestrator notices.
- The system `cargo` may lack `rustfmt` and `clippy`; put `~/.cargo/bin` first on `PATH` to match CI.
- Without decompiled vanilla in `.vendor/`, audits rely on memory. Mark such results unverified.

## Scripts

| Script | Purpose |
|---|---|
| `batch-merge.sh` | batch-merge.sh <name> <pr>...  -- in worktree kab/<name>, apply each upstream PR as a squash commit onto master. |
| `boot.sh` | boot.sh [secs=90]  -- boot debug server in scratch dir, wait for "Done", send `stop`, grep panics/errors. Exit 0 if clean. |
| `brief-pr.sh` | brief-pr.sh <pr>  -- write a self-contained, structured worker brief for landing upstream PR <pr>; prints brief path. |
| `cargo.sh` | cargo.sh <cargo args>  -- the ONLY way agents run cargo: shared warm target, 2 slots, half the cores, waits for free RAM |
| `check.sh` | check.sh [-f] <crate>...  -- default: fmt + cargo check -p (fast). -f: also clippy -D warnings + nextest (before landing |
| `clean-merged.sh` | clean-merged.sh -- remove worktrees + branches of kab/* branches already merged into master. Skips running workers, the  |
| `fix-conflicts.sh` | fix-conflicts.sh <integrate.log> [max]  -- for every "CONFLICT kab/<w> <hash>" line, launch an OpenCode worker (in paral |
| `game-versions.sh` | Prints the supported protocol versions, e.g. `26.2-26.45`. ferrflow appends it |
| `gaps-fixed.sh` | gaps-fixed.sh -- regenerate docs/GAPS_FIXED.md from docs/LEDGER.tsv and the audit notes in docs/tracker/. |
| `gc-targets.sh` | gc-targets.sh  -- delete private per-worker cargo targets and merged worktrees (disk is ~93% full). Keeps /tmp/kab-targe |
| `gen-todo.sh` | gen-todo.sh  -- rebuild docs/TODO.md from docs/queue-*.tsv minus numbers listed in docs/DONE.md. |
| `impl.sh` | impl.sh <issue/pr> <number> <guidance-file>  -- orchestrator-verified mode: launch ONE OpenCode implementer for an item  |
| `int-batch.sh` | int-batch.sh <start> <count> -- integrate lines start..start+count-1 of $KAB_RUNS/unlanded.txt in ONE integrate.sh run. |
| `integrate.sh` | integrate.sh <branch>...  -- cherry-pick finished worker branches onto the persistent worktree `int`, then ONE increment |
| `land.sh` | land.sh <name> [--no-push]  -- rebase kab/<name> on master, verify, fast-forward master, push, drop worktree. |
| `leaderboard.sh` | leaderboard.sh  -- commits per agent (by Co-Authored-By trailer) on master + OpenCode run outcomes. |
| `ledger.sh` | Builds docs/LEDGER.tsv (+ .md): single source of truth for TODO vs DONE of |
| `lib.sh` | Shared helpers. Source, do not run. |
| `next-issue.sh` | next-issue.sh [N] -- pick N untried issues (bugs first), create worktree i<ISSUE>, write brief; prints "<ISSUE> <worktre |
| `next-pr.sh` | next-pr.sh [N]  -- pick the next N (default 1) untried PRs (smallest first), create worktree n<PR>, fetch the PR ref, |
| `oc-agent.sh` | oc-agent.sh <name> <brief.md>  -- start an OpenCode DeepSeek worker in its own worktree, in background. |
| `oc-collect.sh` | oc-collect.sh -- idempotent: turn every worker STATUS line (failed/skipped) into a classified row in docs/OUTCOMES.tsv |
| `oc-finished.sh` | oc-finished.sh [--all] -- table of ONLY the workers that finished since the last call (state: $KAB_RUNS/reported); --all |
| `oc-group.sh` | oc-group.sh <K> -- stdin: "<id> <worktree> <brief>" lines; stdout: same format, K tasks merged into one worker |
| `oc-loop.sh` | oc-loop.sh  -- keep 10 OpenCode workers running forever (refill every 30s). Stop: pkill -f oc-loop.sh |
| `oc-refill.sh` | oc-refill.sh -- fill every free OpenCode slot (KAB_OC_SLOTS, default 20) in PARALLEL: PR workers first (smallest non-dra |
| `oc-status.sh` | oc-status.sh  -- table of OpenCode workers: running/ok/FAIL, plus balance-error detection. |
| `parity.sh` | parity.sh [--no-fetch] -- diff the Pumpkin parity tracker (Pumpkin-Website data.json) against docs/PARITY.tsv. |
| `pr-branch.sh` | pr-branch.sh <upstream-pr-number> <name>  -- fetch upstream PR head into worktree kab/<name> (rebased onto master if cle |
| `pr-triage.sh` | pr-triage.sh <pr>...  -- mechanical triage, no agent needed. Prints "<pr> EMPTY/CLEAN/CONFLICT/CLOSED". |
| `reap-mcp.sh` | reap-mcp.sh -- kill orphaned legal-ar MCP node processes (no live opencode ancestor). opencode global config starts |
| `requeue.sh` | requeue.sh -- make unfinished work eligible again so worker pools never run dry. |
| `sync-upstream.sh` | sync-upstream.sh  -- fetch upstream and report ahead/behind; merges nothing. |
| `tracker-md.sh` | tracker-md.sh [--commit] -- regenerate TRACKER.md from docs/PARITY.tsv, our commits (upstream/master..master) and live w |
| `tracker-refill.sh` | tracker-refill.sh -- keep KAB_TRACKER_SLOTS (default 10) OpenCode workers on parity-tracker slices that no upstream issue. |
| `tracker-watch.sh` | tracker-watch.sh -- keep TRACKER.md current against our commits. Polls every 30s; when master's HEAD or the set of runni |
| `tune-tasks.sh` | tune-tasks.sh -- print tasks-per-issue-worker: grows when recent workers finish fast (<8 min), shrinks when slow (>20 mi |
| `upstream-queue.sh` | upstream-queue.sh prs/issues  -- dump upstream open items as TSV: number, size/lines, title, labels. Retry-safe (gh is f |
| `vendor.sh` | vendor.sh <git-url> [name]  -- shallow-clone a reference repo into .vendor/ (git-ignored, read-only reference). |
| `wt.sh` | wt.sh new <name> [base=master] / warm [N] / rm <name> / ls   -- one worktree+branch (kab/<name>) per worker. |
- `scripts/parity-details.sh` fills the optional `source`, `summary`, `checklist`, `ids` columns of `docs/PARITY.tsv` (source must exist in this tree, checklist keeps only items marked done, summary drops caveat sentences; status/covered/issues/note stay the honest audit).
