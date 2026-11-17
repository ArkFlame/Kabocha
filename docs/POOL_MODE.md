---
name: shared-pool
description: Mass-parallel "edit everything, compile once" workflow. Use for large backlogs (hundreds of small fixes/gaps/refactors) in a compiled project where builds are heavy: run 20-50 cheap OpenCode workers in ONE checkout, each owning a disjoint unit, refill freed slots, then compile once and fix the stacked errors in a single round.
---
# Shared-checkout worker pool (edit all -> compile once -> fix)

Why: worktrees cost minutes each and cause merge conflicts; per-worker compiles are the real cost. Most edits compile, so let errors stack and fix the few at the end ("jump the rocks").

## Setup (per project)
1. Split the backlog into UNITS: one file/doc per unit (e.g. a tracker doc with open rows). Units that name the same source file must not run together (the pool serializes on shared basenames; ignore generic ones like mod.rs).
2. Write a worker brief (`POOL_BRIEF.md`): role CODER, edit only files named in the unit, NO build/test/git, verify against the reference source, small unit test where a seam exists, run the formatter on touched files, record `## Result` (FIXED/VERIFIED/STILL OPEN + reason) in the unit doc, one-line final status.
3. Copy `pool.py` (this dir), set UNIT_GLOB / SKIP / BRIEF / MODELS at the top, run once detached:
   `setsid nohup python3 pool.py 40 10 >/tmp/pool.log 2>&1 &`
   Check `tail pool.log` (STATS line every 5 min); never poll in a tight loop.

## Rules learned
- Lanes: primary model lane (cap 40) + experimental lane (cap 10). A rate-limit/timeout/usage-limit hit requeues the unit, shrinks that lane (experimental halves to 0; primary x3/4), backs off 120s*hits, and the pool stops after repeated hits. Finished units (exit file 0) are skipped on rerun; live workers are adopted when the pool restarts.
- open-design deepseek limits: concurrency 10, 100 rpm (error tier_limit_exceeded): keep that lane at <=6 workers. Non-zero exits must requeue, never count as done.
- Measure before scaling: TTFT/TPS from OpenCode DB (`~/.local/share/opencode/opencode.db`, tables session_v2/session_message; assistant message `time.firstToken - time.created`, `tokens.output+reasoning / (streamed - firstToken)`). Hy4 flash free: TTFT ~3s median, ~200 tok/s; do not raise past ~40-50 concurrent.
- Free tiers hit usage limits under bursts: start with the cap, not 84 failed launches. Verify the model with a one-line `opencode run -m <model> "Reply with exactly: OK"` first; list with `opencode models`.
- Never start the pool with a tight `while true`; it is bounded and exits on POOL DONE.
- Workers must not touch git; the orchestrator commits one batch after the compile.

## After the pool (orchestrator)
1. `git status`, drop garbage. 2. formatter. 3. ONE full compile/lint run (heavy builds: detached, never edit the tree meanwhile). 4. Fix all errors in one round, grouped by file (duplicated helpers from parallel workers are the usual cause), rerun until clean. 5. Run tests once, rerun only failing crates. 6. Commit in a few conventional commits, push.
