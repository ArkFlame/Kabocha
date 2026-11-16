---
name: kabocha-orchestrator-playbook
description: DOs and DON'Ts for the Kabocha orchestrator loop (native + OpenCode workers, batch compile, land). Read at session start before launching workers.
---

# Orchestrator playbook (evidence from live sessions)

## Old technique vs current technique
| Topic | Old (cost) | Current |
|---|---|---|
| Native workers | launched via scripts, orchestrator blocked on logs | `Agent` tool, `subagent_type: general`, `run_in_background: true`; completion arrives as a notification, never poll |
| OpenCode workers | orchestrator waited in `sleep` loops (stuck, memory-heavy watcher shells) | one persistent `Monitor` running `scripts/oc-status.sh` + `scripts/oc-refill.sh` every 60 s; one event per change; re-arm at expiry |
| Slots | refilled by hand, often 3-7 of 10 | refill in the same turn a worker notification arrives |
| Briefs | prose, workers guessed | `scripts/next-pr.sh N` picks PRs, fetches refs, makes worktree + brief, prints ready lines; launch one worker per line |
| Compile | per branch, or by workers | workers never build; orchestrator runs ONE `scripts/integrate.sh <branches>` per batch (warm shared target) |
| Triage | launch worker, find it is empty | `scripts/pr-triage.sh` first: EMPTY = log to DONE.md, CLEAN = batch-merge, CONFLICT = worker |

## DO
- Keep 10 native + 10 OpenCode running; relaunch the moment one ends.
- Fetch the PR ref (`git fetch upstream pull/N/head:refs/kab/pr-N`) before `brief-pr.sh`; the script aborts silently (pipefail) when the ref is missing, and workers then report "brief does not exist".
- Verify the brief file exists (`[ -s ...]`) before launching the worker.
- Batch 10-15 finished branches per `integrate.sh`; fix compile errors on the branch AND `int`, then ff-merge `kab/int` to master and push.
- Strip emoji and trailing spaces from worker commit subjects before landing (workers copy PR titles).
- Co-author is automatic: `scripts/hooks/commit-msg` + `KAB_AGENT` (set by `scripts/oc-agent.sh`). Never hand-write it.
- Record skips (already present) in `docs/DONE.md` with file:line proof; log failures in `docs/QUEUE.md`.
- Expect after-merge compile breaks in: stray `>`/`}` fragments in tests, signature changes (`finalize_spawn`, tick delay width), moved types after crate splits. Grep every caller.

## DON'T
- Don't block on OpenCode (`sleep` loops, `tail -f`). Use Monitor.
- Don't start extra watcher shells: memory pressure killed three at ~36 GB used.
- Don't run `cargo` in workers or parallel builds outside `scripts/cargo.sh` (flock + RAM guard).
- Don't re-launch a PR already in `/tmp/kabocha-runs/tried` without cause.
- Don't cherry-pick merge commits through `integrate.sh` (it cannot); squash them first.
- Don't amend or rewrite history already pushed; fix forward.
- Don't choose a model for native workers: the Agent tool has no model parameter (no Haiku pinning).
- Don't ask the user questions; resolve from vanilla/Paper/upstream references.

## Failure catalogue (symptom -> cause -> fix)
- Worker: "brief does not exist" -> `brief-pr.sh` aborted, ref not fetched -> fetch ref, regenerate, relaunch.
- `expected expression, found '>'` after cherry-pick -> stray fragment in worker squash -> delete line, amend branch.
- Silverfish-style FAILED (#3886): PR built on older goal APIs -> needs a redesign brief, not a re-merge; queue it for a dedicated worker with the master API list.
- oc-refill reports `free_slots` but starts nothing -> all candidates CLEAN/EMPTY/CLOSED this round; run again or widen the candidate filter.

## Self-optimization log (fix inefficiencies as found)
- Repeated candidate python + wt + fetch + brief by hand -> `scripts/next-pr.sh` (one call, verified briefs).
- Failures of ported-but-uncompiled branches are cheaper to drop from a batch (and queue a fix worker) than to debug inside the batch; re-run integrate without them.
- Report skipped/failed PRs to docs/QUEUE.md immediately so the loop never retries them.

## Communication protocol (orchestrator <-> native <-> OpenCode)
Detected problems and the rule that fixes each:
- Workers wrote "merged" for a commit on their own branch -> status vocabulary is `committed|partial|skipped|failed`; only the orchestrator merges to master.
- Partial ports reported as full -> `partial` must name what was left out; log it in docs/QUEUE.md.
- Interim notices ("waiting on its own background work") look like completions -> treat only a final STATUS line as done; workers must not start background jobs (briefs forbid it).
- Missing/empty briefs -> `scripts/next-pr.sh` verifies the brief before a worker is launched.
- Ambiguous "not compiled" -> every report carries `compiled: no`; orchestrator is the only compiler.
- Emoji/trailing spaces copied from PR titles into subjects -> the brief says subject only; strip on landing.
- OpenCode has no notification channel -> track by Monitor events on `oc-status.sh`; its truth is the `.exit` file and branch commits, never its chat output.
- One-way channel: a worker cannot ask questions, so every brief must hold role, context, numbered task, constraints, report format. If a worker fails twice on a PR, requeue it as a redesign brief with master API names listed.
