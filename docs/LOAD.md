# Load and subagent cap (2026-11-16)

## Stop and progress so far
- User capped total subagents at 10. OpenCode default slots 10 (`KAB_OC_SLOTS`), native workers count toward the same cap.
- Stopped 7 surplus OpenCode workers (i3382 i3385 i3512 i3514 i3515 i3518 i3742); kept the 10 with commits/newest.
- Earlier: killed 20 hung workers (ECONNRESET, 68-81 min idle) and 18 stale ones; oc-agent.sh now kills the process group, idle watchdog 300 s, one retry.
- Landed on master: batches 14-18 plus batch 25 (13 branches); upstream merge (2 commits, leash lock order in entity/mod.rs) committed locally, workspace check pending before push.
- Master about 311 ahead and 2 behind upstream at last sync.

## What was churning the machine (snapshot 19:03, 12 cores, 38 GB)
| Source | Evidence | Verdict |
|---|---|---|
| Load average 23 / 32 / 39 / 43 (1/5/15 min) | `uptime` | oversubscribed: up to 20 OpenCode workers + cargo + git |
| opencode workers | 9 procs, ~2.2 GB, ~90% CPU; one at 1.3 GB, 1 h old | ours; capped now |
| cargo/rustc workspace check | cold build from the main checkout (different path than `int`), wasmtime and deps | ours; run checks from `int` so the warm target is reused |
| git | 9 procs; my `git cherry master <branch>` loop over ~230 branches | ours; cache the unlanded list instead of rescanning |
| java x7 (74% CPU), python x74, chrome x72 (2.4 GB), helium x20 | long-running, other projects and browsers (e.g. 87 h java, 85 h media_card_proof.py) | not Kabocha; leave alone, but they eat headroom |
| RAM | 32 of 38 GB used, 6 GB available | tight; below the 7 GB guard |

## Guardrails (applied)
1. `oc-refill.sh` pauses when load > 18 or available RAM < 7 GB (`KAB_MAX_LOAD`, `KAB_MIN_AVAIL_GB`).
2. Total subagent cap 10 (was 10+10).
3. Run `integrate.sh`/cargo from the main checkout only through `scripts/cargo.sh` (flock + RAM guard); one build at a time.
4. Do not rescan all branches with `git cherry` repeatedly; compute the unlanded list once per batch.
5. Kill workers by `.pid` file, never `pkill -f` patterns.
