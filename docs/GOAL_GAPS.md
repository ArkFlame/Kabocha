GOAL: Close every remaining Kabocha parity gap with 5 Claude subagents that are ALWAYS running. Never stop until zero gaps are open.

STANDING RULE (highest priority)
- At all times exactly 5 subagents (Agent tool, subagent_type general, run_in_background true) are working on gaps. The moment ANY ends (completed, failed, blocked, timed out), launch its replacement in the same turn, before doing anything else. Never wait for the other four. Never let the count drop below 5. Check with ListAgents at the start of every turn and top up to 5.
- Stopping, summarizing, or reporting while gaps remain is a failure. Only the DONE condition ends the loop.

GAP SOURCE
- Open gaps = rows in docs/tracker/*.md that are not marked FIXED (S, M, then L), plus docs/TODO_2.md and TRACKER.md rows not AUDITED/LANDED/DONE. Count them with scripts/gaps-fixed.sh (extend it to count FIXED vs STILL OPEN) and keep docs/GAPS_FIXED.md current.
- Pick work by size: S first, then M, then L. Give each worker a disjoint set of files (one note, or a few related notes) so workers never touch the same file.
- If a gap needs data that does not exist (assets/ lacks it, needs the Extractor), mark it BLOCKED-DATA with the reason in the note and move on. If a note's gap is only "manual review", re-check the code once against vanilla; if it matches, mark it VERIFIED and close it.

WORKER BRIEF (every worker)
- Role: CODER. Own worktree: scripts/wt.sh new <name>, branch kab/<name>. Never cargo, never merge, never push, never touch other worktrees.
- Task: fix the listed gaps in vanilla-parity terms (decompiled reference in .vendor/ or tmp/mc-proof/ref if present; else vanilla knowledge, marked UNVERIFIED). Smallest correct change. No unwrap/expect/panic, tracing not println, no em dashes, never hand-edit crates/pumpkin-data/src/generated.
- Tests: one small MVP automated test per fix that fails without it (bug repro, boundary, codec round trip). No formula-mirroring or over-mocked tests. Kabocha is fully AI automated: tests are the gate, not human review.
- Update the note: mark each gap FIXED (commit hash) or STILL OPEN with reason. Commit subject-only conventional.
- Report one line: STATUS, gaps fixed, files, tests added.

ORCHESTRATOR LOOP (you)
1. Keep 5 workers running. On each completion notification, immediately launch the next.
2. Every 10 finished workers (or when the queue is empty): scripts/int-batch.sh (one workspace check, fix compile errors in kab/int, dedupe the Bedrock item_stack_request helpers at source), ff-merge to master, git push, scripts/clean-merged.sh. Run scripts/cargo.sh clippy and the touched crates' tests; fix failures forward. Launching workers is never blocked by integration: run integration in the background.
3. Run cargo fmt before each push. Keep clippy -D warnings, typos and fmt clean (CI must stay green). Master only, push each verified slice at once.
4. After each batch run scripts/tracker-md.sh, scripts/ledger.sh, scripts/gaps-fixed.sh and scripts/parity.sh; keep docs/PARITY.tsv covered column and docs/TODO_2.md current.
5. Never ask the user. Decide from vanilla, Paper, .vendor/ and docs, record the precedent in the note.

DONE WHEN (all true)
- Zero STILL OPEN gaps in docs/tracker/*.md and docs/TODO_2.md (all FIXED, VERIFIED or BLOCKED-DATA with a reason).
- Every fix has its small test and the tests pass; cargo fmt, clippy -D warnings, typos and nextest are green locally.
- Ledger has 0 TODO/FAILED/BLOCKED/COMMITTED-UNLANDED; TRACKER.md 100%; docs/GAPS_FIXED.md, docs/DONE.md, docs/TODO.md updated.
- git status clean, origin/master == master, only master branch, no extra worktrees.
